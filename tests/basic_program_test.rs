use etd3200 as e;
use std::fs;
use std::io::Read;
use e::emulator::UnknownCpu;

#[test]
fn can_make_cpu() {
    let cpu = e::emulator::Cpu::new();
}

#[test]
fn can_load_program() {
    let mut cpu = e::emulator::Cpu::new_blank();
    cpu.program_counter = 0;
    let program = e::program_loader::parse_machine_code(
        fs::read_to_string("sample_code/1-10.mc").unwrap()
        );
    for (i, instruction) in program.iter().enumerate() {
        cpu.load_instruction((i*4) as u8, instruction);
    }
    let mut limit = 100;
    let icpu = loop {
        limit -= 1;
        if limit == 0 {
            panic!("Cpu stuck in a loop {cpu}");
        }
        //println!("{}", cpu);
        cpu = match cpu.clock() {
            UnknownCpu::Ok(cpu) => cpu,
            UnknownCpu::Inter(cpu) => break cpu,
        }
    };
    println!("{}", icpu);
    let memory = icpu.memory;
    for i in 0..11 {
        assert_eq!(i, memory.read(i + 64).unwrap());
    }

}
