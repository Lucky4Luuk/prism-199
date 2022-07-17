use std::io::Read;
use wasmer::{Store, Module, Instance, Value, NativeFunc, imports};

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
        let import_object = imports! {};
        let instance = Instance::new(&module, &import_object).expect("Failed to create OS wasm instance!");

        // First we have to copy our slice into the VM memory
        // This way it becomes accessible to our code running in the wasmer VM
        let memory = instance.exports.get_memory("memory").expect("Failed to get memory!");
        // let set_at: NativeFunc<(i32, i32), ()> = instance.exports.get_native_function("set_at").expect("Failed to get set_at function!");
        let buf_mem_addr = memory.data_size() as u32;
        println!("mem_addr: {}", buf_mem_addr);
        memory.grow(1).expect("Failed to grow memory!");
        // set_at.call(mem_addr, &[0u8; 168*72*4]);
        // unsafe { memory.view().copy_from(&[0u8; 168*72*4]) };
        // for (byte, cell) in [0u8; 168*72*4].bytes().zip(memory.view()[0..(168*72*4)].iter())
        // {
        //     cell.set(byte);
        // }
        let buf = [0u8; 168*72*4];
        memory.view()[buf_mem_addr as usize .. (buf_mem_addr as usize + 168*72*4)].iter().enumerate().for_each(|(i, c)| c.set(buf[i]));

        Self {
            module: module,
            instance: instance,

            buf_mem_addr: buf_mem_addr,
        }
    }

    pub fn draw(&mut self, info: crate::FrameInfo) {
        // The framebuffer slice exists in the VM too, so we can use it to call the draw function
        let func: NativeFunc<u32, ()> = self.instance.exports.get_native_function("draw_unsafe").expect("Failed to get draw_unsafe function!");
        func.call(self.buf_mem_addr).expect("Failed to call draw_unsafe function!");

        // After doing so, we must read back the slice from the VM's memory
        // We need to do this, so we can actually see the data the VM has changed and render it
        let memory = self.instance.exports.get_memory("memory").expect("Failed to get memory!");
        let buf_view = memory.view().subarray(self.buf_mem_addr, self.buf_mem_addr + 168*72*4);
        let buf: Vec<u8> = buf_view[..].iter().map(|c| c.get()).collect();
        info.buf.copy_from_slice(&buf);
    }
}
