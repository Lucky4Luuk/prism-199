use wasmer::*;

use crate::vfs::VFS;

pub struct Runtime {
    module: Module,
    instance: Instance,

    buf_mem_addr: u32,
}

impl Runtime {
    pub fn new(os_path: &str) -> Self {
        let wasm_bytes = std::fs::read(os_path).expect("File does not exist!");

        let store = Store::default();
        let module = Module::new(&store, wasm_bytes).expect("OS code does not compile!");
        let env = Env { memory: LazyInit::new() };
        let import_object = imports! {
            "env" => {
                "fs_create_folder" => Function::new_native_with_env(&store, env, fs_create_folder)
            }
        };
        let instance = Instance::new(&module, &import_object).expect("Failed to create OS wasm instance!");

        // First we have to copy our slice into the VM memory
        // This way it becomes accessible to our code running in the wasmer VM
        let memory = instance.exports.get_memory("memory").expect("Failed to get memory!");
        let buf_mem_addr = memory.data_size() as u32;
        println!("mem_addr: {}", buf_mem_addr);
        memory.grow(3).expect("Failed to grow memory!");
        let buf = [0u8; crate::BUFFER_LEN];
        memory.view()[buf_mem_addr as usize .. (buf_mem_addr as usize + crate::BUFFER_LEN)].iter().enumerate().for_each(|(i, c)| c.set(buf[i]));

        Self {
            module: module,
            instance: instance,

            buf_mem_addr: buf_mem_addr,
        }
    }

    pub fn tick(&mut self, info: crate::FrameInfo, input: u64, delta_s: f32) {
        // The framebuffer slice exists in the VM too, so we can use it to call the draw function
        let func: NativeFunc<(u32, u64, f32), ()> = self.instance.exports.get_native_function("tick").expect("Failed to get tick function!");
        func.call(self.buf_mem_addr, input, delta_s).expect("Failed to call tick function!");

        // After doing so, we must read back the slice from the VM's memory
        // We need to do this, so we can actually see the data the VM has changed and render it
        let memory = self.instance.exports.get_memory("memory").expect("Failed to get memory!");
        let buf_view = memory.view().subarray(self.buf_mem_addr, self.buf_mem_addr + crate::BUFFER_LEN as u32);
        let buf: Vec<u8> = buf_view[..].iter().map(|c| c.get()).collect();
        info.buf.copy_from_slice(&buf);
    }
}

#[derive(WasmerEnv, Clone)]
pub struct Env {
    #[wasmer(export)]
    memory: LazyInit<Memory>,
}

fn fs_create_folder(env: &Env, path_str_ptr: WasmPtr<u8, Array>, path_str_len: u32) {
    let path = unsafe { path_str_ptr.get_utf8_str(&env.memory.get_unchecked(), path_str_len) };
    todo!();
}
