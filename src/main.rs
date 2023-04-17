use setjmp::sigjmp_buf;
use wasmedge_sdk::WasmValue;

scoped_tls::scoped_thread_local!(static JMP_BUF:setjmp::sigjmp_buf);

fn main() {
    println!("Hello, world!");
    let common_cfg = wasmedge_sdk::config::CommonConfigOptions::new();
    let config = wasmedge_sdk::config::ConfigBuilder::new(common_cfg);
    let host_cfg = wasmedge_sdk::config::HostRegistrationConfigOptions::default().wasi(true);
    let config_builder = config.with_host_registration_config(host_cfg);

    let wasi = wasmedge_sdk::ImportObjectBuilder::new()
        .build_as_wasi(None, None, None)
        .unwrap();

    // let vm = wasmedge_sdk::Vm::new(Some(config_builder.build().unwrap())).unwrap();
    let vm = wasmedge_sdk::Vm::new(None).unwrap();
    let vm = vm.register_import_module(wasi).unwrap();

    let args: Vec<WasmValue> = vec![];
    unsafe {
        let mut jmp_buf: setjmp::sigjmp_buf = std::mem::zeroed();
        let i = setjmp::sigsetjmp(&mut jmp_buf, 1);
        if i == 0 {
            JMP_BUF.set(&jmp_buf, || {
                libc::signal(libc::SIGALRM, sign_handler as libc::sighandler_t);
                libc::alarm(2);
                let r = vm
                    .run_func_from_file("demo_wasm.wasm", "_start", args)
                    .unwrap();
                println!("{:?}", r)
            });
        }
        println!("exit!")
    }
}

fn sign_handler(s: i32) {
    println!("sign {s}");
    if JMP_BUF.is_set() {
        println!("jmp");
        let buf = JMP_BUF.with(|buf| buf as *const sigjmp_buf);
        unsafe {
            setjmp::siglongjmp(buf as usize as *mut sigjmp_buf, 1);
        }
    }
}
