//! This file is mainly to assure specific issues are working well
use anyhow::Result;
use wasmer::FunctionEnv;
use wasmer::*;

/// Corruption of WasmerEnv when using call indirect.
///
/// Note: this one is specific to Singlepass, but we want to test in all
/// available compilers.
///
/// https://github.com/wasmerio/wasmer/issues/2329
#[compiler_test(issues)]
fn issue_2329(mut config: crate::Config) -> Result<()> {
    let mut store = config.store();

    #[derive(Clone, Default)]
    pub struct Env {
        memory: Option<Memory>,
    }

    impl Env {
        pub fn new() -> Self {
            Self { memory: None }
        }
    }

    pub fn read_memory(mut ctx: FunctionEnvMut<Env>, guest_ptr: u32) -> u32 {
        dbg!(ctx.data_mut().memory.as_ref());
        dbg!(guest_ptr);
        0
    }

    let wat = r#"
    (module
        (type (;0;) (func (param i32) (result i32)))
        (type (;1;) (func))
        (type (;2;) (func (param i32 i32) (result i32)))
        (import "env" "__read_memory" (func $__read_memory (type 0)))
        (func $read_memory (type 1)
          (drop
            (call $_ZN5other8dispatch17h053cb34ef5d0d7b0E
              (i32.const 1)
              (i32.const 2)))
          (drop
            (call $__read_memory
              (i32.const 1))))
        (func $_ZN5other8dispatch17h053cb34ef5d0d7b0E (type 2) (param i32 i32) (result i32)
          (call_indirect (type 0)
            (local.get 1)
            (local.get 0)))
        (table (;0;) 2 2 funcref)
        (memory (;0;) 16)
        (global (;0;) (mut i32) (i32.const 1048576))
        (global (;1;) i32 (i32.const 1048576))
        (global (;2;) i32 (i32.const 1048576))
        (export "memory" (memory 0))
        (export "read_memory" (func $read_memory))
        (export "__data_end" (global 1))
        (export "__heap_base" (global 2))
        (elem (;0;) (i32.const 1) func $__read_memory))
    "#;
    let module = Module::new(&store, wat)?;
    let env = Env::new();
    let mut env = FunctionEnv::new(&mut store, env);
    let imports: Imports = imports! {
        "env" => {
            "__read_memory" => Function::new_typed_with_env(
                &mut store,
                &env,
                read_memory
            ),
        }
    };
    let instance = Instance::new(&mut store, &module, &imports)?;
    instance
        .exports
        .get_function("read_memory")?
        .call(&mut store, &[])?;
    Ok(())
}

#[compiler_test(issues)]
fn call_with_static_data_pointers(mut config: crate::Config) -> Result<()> {
    let mut store = config.store();

    #[derive(Clone)]
    pub struct Env {
        memory: Option<Memory>,
    }

    pub fn banana(
        mut ctx: FunctionEnvMut<Env>,
        a: u64,
        b: u64,
        c: u64,
        d: u64,
        e: u64,
        f: u64,
        g: u64,
        h: u64,
    ) -> u64 {
        println!("{:?}", (a, b, c, d, e, f, g, h));
        let mut buf = vec![0; d as usize];
        let memory = ctx.data_mut().memory.as_ref().unwrap().clone();
        memory.view(&ctx).read(e, &mut buf).unwrap();
        let input_string = std::str::from_utf8(&buf).unwrap();
        assert_eq!(input_string, "bananapeach");
        0
    }

    pub fn mango(ctx: FunctionEnvMut<Env>, a: u64) {}

    pub fn chaenomeles(ctx: FunctionEnvMut<Env>, a: u64) -> u64 {
        0
    }

    pub fn peach(ctx: FunctionEnvMut<Env>, a: u64, b: u64) -> u64 {
        0
    }

    pub fn gas(ctx: FunctionEnvMut<Env>, a: u32) {}

    let wat = r#"
    (module
      (type (;0;) (func (param i64)))
      (type (;1;) (func (param i64) (result i64)))
      (type (;2;) (func (param i64 i64) (result i64)))
      (type (;3;) (func (param i64 i64 i64 i64 i64 i64 i64 i64) (result i64)))
      (type (;4;) (func))
      (import "env" "mango" (func (;0;) (type 0)))
      (import "env" "chaenomeles" (func (;1;) (type 1)))
      (import "env" "peach" (func (;2;) (type 2)))
      (import "env" "banana" (func (;3;) (type 3)))
      (import "env" "memory" (memory (;0;) 1024 2048))
      (func (;4;) (type 4)
        (local i32 i64)
        global.get 0
        i32.const 32
        i32.sub
        local.tee 0
        global.set 0
        local.get 0
        i32.const 8
        i32.add
        i64.const 0
        i64.store
        local.get 0
        i64.const 0
        i64.store
        i64.const 0
        call 0
        i64.const 0
        call 1
        local.set 1
        local.get 0
        i64.const 0
        i64.store offset=24
        local.get 0
        i64.const 0
        i64.store offset=16
        i64.const 0
        i64.const 0
        call 2
        local.get 1
        local.get 0
        i64.extend_i32_u
        i64.const 11
        i32.const 1048576
        i64.extend_i32_u
        i64.const 0
        i64.const 0
        local.get 0
        i32.const 16
        i32.add
        i64.extend_i32_u
        call 3
        return)
      (global (;0;) (mut i32) (i32.const 1048576))
      (global (;1;) i32 (i32.const 1048587))
      (global (;2;) i32 (i32.const 1048592))
      (global (;3;) (mut i32) (i32.const 0))
      (export "memory" (memory 0))
      (export "repro" (func 4))
      (export "__data_end" (global 1))
      (export "__heap_base" (global 2))
      (data (;0;) (i32.const 1048576) "bananapeach"))
    "#;

    let module = Module::new(&store, wat)?;
    let env = Env { memory: None };
    let mut env = FunctionEnv::new(&mut store, env);
    let memory = Memory::new(
        &mut store,
        MemoryType::new(Pages(1024), Some(Pages(2048)), false),
    )
    .unwrap();
    env.as_mut(&mut store).memory = Some(memory.clone());
    let mut exports = Exports::new();
    exports.insert("memory", memory);
    exports.insert(
        "banana",
        Function::new_typed_with_env(&mut store, &env, banana),
    );
    exports.insert(
        "peach",
        Function::new_typed_with_env(&mut store, &env, peach),
    );
    exports.insert(
        "chaenomeles",
        Function::new_typed_with_env(&mut store, &env, chaenomeles),
    );
    exports.insert(
        "mango",
        Function::new_typed_with_env(&mut store, &env, mango),
    );
    exports.insert("gas", Function::new_typed_with_env(&mut store, &env, gas));
    let mut imports = Imports::new();
    imports.register_namespace("env", exports);
    let instance = Instance::new(&mut store, &module, &imports)?;
    instance
        .exports
        .get_function("repro")?
        .call(&mut store, &[])?;
    Ok(())
}

/// Exhaustion of GPRs when calling a function with many floating point arguments
///
/// Note: this one is specific to Singlepass, but we want to test in all
/// available compilers.
#[compiler_test(issues)]
fn regression_gpr_exhaustion_for_calls(mut config: crate::Config) -> Result<()> {
    let mut store = config.store();
    let wat = r#"
        (module
          (type (;0;) (func (param f64) (result i32)))
          (type (;1;) (func (param f64 f64 f64 f64 f64 f64)))
          (func (;0;) (type 0) (param f64) (result i32)
            local.get 0
            local.get 0
            local.get 0
            local.get 0
            f64.const 0
            f64.const 0
            f64.const 0
            f64.const 0
            f64.const 0
            f64.const 0
            f64.const 0
            i32.const 0
            call_indirect (type 0)
            call_indirect (type 1)
            drop
            drop
            drop
            drop
            i32.const 0)
          (table (;0;) 1 1 funcref))
    "#;
    let mut env = FunctionEnv::new(&mut store, ());
    let module = Module::new(&store, wat)?;
    let imports: Imports = imports! {};
    let instance = Instance::new(&mut store, &module, &imports)?;
    Ok(())
}

#[compiler_test(issues)]
fn test_popcnt(mut config: crate::Config) -> Result<()> {
    let mut store = config.store();
    let mut env = FunctionEnv::new(&mut store, ());
    let imports: Imports = imports! {};

    let wat = r#"
    (module
        (func $popcnt_i32 (export "popcnt_i32") (param i32) (result i32)
            local.get 0
            i32.popcnt
        )
        (func $popcnt_i64 (export "popcnt_i64") (param i64) (result i64)
            local.get 0
            i64.popcnt
        )
    )"#;

    let module = Module::new(&store, wat)?;
    let instance = Instance::new(&mut store, &module, &imports)?;

    let popcnt_i32 = instance.exports.get_function("popcnt_i32").unwrap();
    let popcnt_i64 = instance.exports.get_function("popcnt_i64").unwrap();

    let get_next_number_i32 = |mut num: i32| {
        num ^= num << 13;
        num ^= num >> 17;
        num ^= num << 5;
        num
    };
    let get_next_number_i64 = |mut num: i64| {
        num ^= num << 34;
        num ^= num >> 40;
        num ^= num << 7;
        num
    };

    let mut num = 1;
    for _ in 1..10000 {
        let result = popcnt_i32.call(&mut store, &[Value::I32(num)]).unwrap();
        assert_eq!(&Value::I32(num.count_ones() as i32), result.get(0).unwrap());
        num = get_next_number_i32(num);
    }

    let mut num = 1;
    for _ in 1..10000 {
        let result = popcnt_i64.call(&mut store, &[Value::I64(num)]).unwrap();
        assert_eq!(&Value::I64(num.count_ones() as i64), result.get(0).unwrap());
        num = get_next_number_i64(num);
    }

    Ok(())
}
