use wasmtime::*;
use wasmtime_wasi::WasiCtx;

fn spawn_runtime(caller: Caller<'_, Env>, ptr: u32, len: u32) {
    println!("it worked!");
}

struct Env {
    wasi: WasiCtx,
}

pub struct Runtime {
    store: Store<Env>,
    instance: Instance,

    buf_mem_addr: u32,
}

impl Runtime {
    pub fn new(os_path: &str) -> Self {
        let wasm_bytes = std::fs::read(os_path).expect("File does not exist!");

        let engine = Engine::default();
        let module = Module::new(&engine, wasm_bytes).unwrap();
        let mut linker = Linker::new(&engine);
        linker.func_wrap("env", "spawn_runtime", |caller: Caller<'_, Env>, ptr: i32, len: i32| spawn_runtime(caller, ptr as u32, len as u32)).unwrap();
        wasmtime_wasi::add_to_linker(&mut linker, |s: &mut Env| &mut s.wasi).unwrap();
        // let dir = wasmtime_wasi::Dir::from_std_file(std::fs::OpenOptions::new().read(true).write(true).create(true).open("./disk/").unwrap());
        let wasi = wasmtime_wasi::WasiCtxBuilder::new()
            .inherit_stdio()
            // .preopened_dir(dir, "/").unwrap()
            .build();
        let mut store = Store::new(&engine, Env {
            wasi: wasi,
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
        }
    }

    pub fn tick(&mut self, info: crate::FrameInfo, input: u64, delta_s: f32) {
        // The framebuffer slice exists in the VM too, so we can use it to call the draw function
        let func = self.instance.get_typed_func::<(u64, f32), (), _>(&mut self.store, "tick").expect("Failed to get tick function!");
        func.call(&mut self.store, (input, delta_s)).expect("Failed to call tick function!");

        // After doing so, we must read back the slice from the VM's memory
        // We need to do this, so we can actually see the data the VM has changed and render it
        let memory = self.instance.get_memory(&mut self.store, "memory").expect("Failed to get memory!");
        // let buf_view = memory.data_mut(&mut self.store).subarray(self.buf_mem_addr, self.buf_mem_addr + crate::BUFFER_LEN as u32);
        let buf: Vec<u8> = memory.data_mut(&mut self.store)[self.buf_mem_addr as usize .. (self.buf_mem_addr as usize + crate::BUFFER_LEN)].iter().map(|c| *c).collect();
        info.buf.copy_from_slice(&buf);
    }
}
