use wasmtime::*;
use wasmtime_wasi::WasiCtx;

/// This function returns the program index + 1
/// If it returns 0, an error has occured. There's currently no way to see what the error was
fn spawn_runtime(mut caller: Caller<'_, Env>, ptr: u64, len: u64) -> u64 {
    let exp = caller.get_export("memory");
    match exp {
        Some(Extern::Memory(mem)) => {
            let mut data_buf = vec![0u8; len as usize];
            let err = mem.read(&mut caller, ptr as usize, &mut data_buf);
            if err.is_err() { return 0; }
            let mut store = caller.as_context_mut();
            let env = store.data_mut();
            let runtime = Runtime::from_bytes(&data_buf);
            env.children.push(runtime);
            env.children.len() as u64 //Return current program index + 1
        },
        _ => 0,
    }
}

pub struct Env {
    wasi: WasiCtx,
    pub children: Vec<Runtime>,
}

pub struct Runtime {
    pub store: Store<Env>,
    instance: Instance,

    buf_mem_addr: u32,
    is_done: bool,
}

impl Runtime {
    pub fn new(os_path: &str) -> Self {
        let wasm_bytes = std::fs::read(os_path).expect("File does not exist!");
        Self::from_bytes(&wasm_bytes)
    }

    pub fn from_bytes(wasm_bytes: &[u8]) -> Self {
        let engine = Engine::default();
        let module = Module::new(&engine, wasm_bytes).unwrap();
        let mut linker = Linker::new(&engine);
        linker.func_wrap("env", "spawn_runtime", |caller: Caller<'_, Env>, ptr: u64, len: u64| spawn_runtime(caller, ptr, len)).unwrap();
        wasmtime_wasi::add_to_linker(&mut linker, |s: &mut Env| &mut s.wasi).unwrap();
        let dir = wasmtime_wasi::Dir::open_ambient_dir("disk", wasmtime_wasi::sync::ambient_authority()).expect("Failed to preopen disk directory!");
        let wasi = wasmtime_wasi::WasiCtxBuilder::new()
            .inherit_stdio()
            .preopened_dir(dir, "/").expect("Failed to preopen directory!")
            .build();
        let mut store = Store::new(&engine, Env {
            wasi: wasi,
            children: Vec::new(),
        });
        let instance = linker.instantiate(&mut store, &module).unwrap();

        // First we have to copy our slice into the VM memory
        // This way it becomes accessible to our code running in the wasmer VM
        let memory = instance.get_memory(&mut store, "memory").expect("Failed to get memory!");
        // let buf_mem_addr = memory.data_size() as u32;
        let buf_mem_addr = 0x80;
        println!("buf_mem_addr: {}", buf_mem_addr);
        memory.grow(&mut store, 3).expect("Failed to grow memory!");
        let buf = [0u8; crate::BUFFER_LEN];
        memory.data_mut(&mut store)[buf_mem_addr as usize .. (buf_mem_addr as usize + crate::BUFFER_LEN)].iter_mut().enumerate().for_each(|(i, c)| *c = buf[i]);

        Self {
            store: store,
            instance: instance,

            buf_mem_addr: buf_mem_addr,
            is_done: false,
        }
    }

    pub fn tick(&mut self, frame: &mut [u8], input: u64, delta_s: f32) -> u32 {
        // The framebuffer slice exists in the VM too, so we can use it to call the draw function
        let func = self.instance.get_typed_func::<(u64, f32), u32, _>(&mut self.store, "tick").expect("Failed to get tick function!");
        let tick_result = func.call(&mut self.store, (input, delta_s)).expect("Failed to call tick function!");

        // After doing so, we must read back the slice from the VM's memory
        // We need to do this, so we can actually see the data the VM has changed and render it
        let memory = self.instance.get_memory(&mut self.store, "memory").expect("Failed to get memory!");
        // let buf_view = memory.data_mut(&mut self.store).subarray(self.buf_mem_addr, self.buf_mem_addr + crate::BUFFER_LEN as u32);
        let buf: Vec<u8> = memory.data_mut(&mut self.store)[self.buf_mem_addr as usize .. (self.buf_mem_addr as usize + crate::BUFFER_LEN)].iter().map(|c| *c).collect();
        frame.copy_from_slice(&buf);

        for i in 0..self.store.data_mut().children.len() {
            if self.store.data_mut().children[i].tick(frame, input, delta_s) > 0 {
                self.store.data_mut().children.remove(i);
            }
        }

        tick_result
    }
}
