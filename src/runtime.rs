// use std::io::Cursor;
use std::sync::{Arc, RwLock};
use crate::rw_cursor::RwCursor;

use wasmtime::*;
use wasmtime_wasi::WasiCtx;
use wasmtime_wasi::sync::stdio::stdout;
use wasi_common::pipe::WritePipe;

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
            let runtime = Runtime::from_bytes(&data_buf, Some(env.out.clone()));
            env.children.push(runtime);
            env.children.len() as u64 //Return current program index + 1
        },
        _ => 0,
    }
}

fn read_stdout(mut caller: Caller<'_, Env>, buf_ptr: u64, buf_len: u64) -> u64 {
    use std::io::Read;
    if let Some(Extern::Memory(mem)) = caller.get_export("memory") {
        let mut store = caller.as_context_mut();
        let mut buf = vec![0u8; buf_len as usize];
        let bytes_read = {
            let env = store.data();
            let mut lock = env.out_buf.as_ref().write().unwrap();
            lock.read(&mut buf).unwrap_or(0)
        };
        if bytes_read > 0 {
            mem.data_mut(&mut store)[buf_ptr as usize .. (buf_ptr + buf_len) as usize].copy_from_slice(&buf);
        }
        bytes_read as u64
    } else {
        0
    }
}

pub struct Env<'buf> {
    wasi: WasiCtx,
    pub children: Vec<Runtime<'buf>>,
    pub out: WritePipe<RwCursor<Vec<u8>>>,
    out_buf: Arc<RwLock<RwCursor<Vec<u8>>>>,
}

pub struct Runtime<'env> {
    pub store: Store<Env<'env>>,
    instance: Instance,

    buf_mem_addr: u32,
}

impl<'env> Runtime<'env> {
    pub fn new(os_path: &str, output: Option<WritePipe<RwCursor<Vec<u8>>>>) -> Self {
        let wasm_bytes = std::fs::read(os_path).expect("File does not exist!");
        Self::from_bytes(&wasm_bytes, output)
    }

    pub fn from_bytes(wasm_bytes: &[u8], output: Option<WritePipe<RwCursor<Vec<u8>>>>) -> Self {
        let engine = Engine::default();
        let module = Module::new(&engine, wasm_bytes).unwrap();

        let mut linker = Linker::new(&engine);
        linker.func_wrap("env", "spawn_runtime", |caller: Caller<'_, Env>, ptr: u64, len: u64| spawn_runtime(caller, ptr, len)).unwrap();
        linker.func_wrap("env", "read_stdout", |caller: Caller<'_, Env>, buf_ptr: u64, buf_len: u64| read_stdout(caller, buf_ptr, buf_len)).unwrap();

        wasmtime_wasi::add_to_linker(&mut linker, |s: &mut Env| &mut s.wasi).unwrap();
        let dir = wasmtime_wasi::Dir::open_ambient_dir("disk", wasmtime_wasi::sync::ambient_authority()).expect("Failed to preopen disk directory!");
        let mut wasi = wasmtime_wasi::WasiCtxBuilder::new().preopened_dir(dir, "/").expect("Failed to preopen directory!");
        if let Some(pipe) = output {
            wasi = wasi.stdout(Box::new(pipe));
        } else {
            wasi = wasi.stdout(Box::new(stdout()));
        }
        let out_buf = Arc::new(RwLock::new(RwCursor::new(Vec::new())));
        let mut store = Store::new(&engine, Env {
            wasi: wasi.build(),
            children: Vec::new(),
            out: WritePipe::from_shared(out_buf.clone()),
            out_buf: out_buf,
        });
        let instance = linker.instantiate(&mut store, &module).unwrap();

        // First we have to copy our slice into the VM memory
        // This way it becomes accessible to our code running in the wasmer VM
        let memory = instance.get_memory(&mut store, "memory").expect("Failed to get memory!");
        let buf_mem_addr = 0x80;
        memory.grow(&mut store, 3).expect("Failed to grow memory!");
        let buf = [0u8; crate::BUFFER_LEN];
        memory.data_mut(&mut store)[buf_mem_addr as usize .. (buf_mem_addr as usize + crate::BUFFER_LEN)].iter_mut().enumerate().for_each(|(i, c)| *c = buf[i]);

        Self {
            store: store,
            instance: instance,

            buf_mem_addr: buf_mem_addr,
        }
    }

    pub fn tick(&mut self, frame: &mut [u8], input: u64, delta_s: f32) -> u32 {
        let func = self.instance.get_typed_func::<(u64, f32), u32, _>(&mut self.store, "tick").expect("Failed to get tick function!");
        let tick_result = func.call(&mut self.store, (input, delta_s)).expect("Failed to call tick function!");

        // After calling, we must read the framebuffer slice from the VM's memory
        // We need to do this, so we can actually see the data the VM has changed and render it
        let memory = self.instance.get_memory(&mut self.store, "memory").expect("Failed to get memory!");
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
