#![allow(dead_code)]
use crate::emulator::Instruction;
use rand;
use rand::prelude::*;

fn parse_machine_code(program:String) -> Vec<Instruction> {
    let result =
        program
        .lines()
        .map(|line| {
            println!("{}", line);
                line.chars()
                .take_while(|c| *c != '#')
                .take_while(|c| *c != ':')
                .filter(|c| *c == '1' || *c == '0')
                .collect::<String>()
        })
    .filter(|result| !result.is_empty())
    .map(|num| if num.len() == 32 {num} else {panic!("Machine Code length wrong {}", num.len())})
    .map(|num| {println!("||{}||", num); num})
    .map(|value| u32::from_str_radix(&value, 2))
    .filter(|result| result.is_ok())
    .map(|bi| bi.unwrap())
    .map(|bi| Instruction::decode(bi)) 
    .collect();
    return result;
}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn skips_comments() {
        let program = String::from("#This is a comment");
        let output = parse_machine_code(program);
        assert!(output.is_empty());
    }

    #[test]
    fn skips_comments_at_end_of_lies() {
        let program = String::from("00001000000000000000000000000000 # What is this comment?");
        let output = parse_machine_code(program);
        assert_eq!(1, output.len());
    }

    #[test]
    fn can_handle_empty_lines() {
        let program = String::from("00001000000000000000000000000000 # What is this comment?\n\n");
        let output = parse_machine_code(program);
        assert_eq!(1, output.len());
    }

    #[test]
    fn can_handle_labels() {
        let program = String::from(":Label01\n00001000000000000000000000000000 # What is this comment?\n\n");
        let output = parse_machine_code(program);
        assert_eq!(1, output.len());
    }

    #[test]
    fn does_load_single_instruction() {
        // Simple interrupt instruction
        let program = String::from("00001000000000000000000000000000");
        let output = parse_machine_code(program);
        assert_eq!(1, output.len());
    }

    #[test]
    fn does_load_single_instruction_correctly() {
        // Simple interrupt instruction
        let instruction = Instruction::decode(0b00001000000000000000000000000000);
        let program = String::from("00001000000000000000000000000000");
        let output = parse_machine_code(program);

        assert_eq!(Some(&instruction), output.first());
    }

    #[test]
    fn does_test_all_instructions() {
        let mut instruction = Instruction::decode(0);
        let mut program = format!("0");
        let mut output = Vec::new();
        let mut rng = rand::thread_rng();
        for _ in 0..1000 {
            let i = rng.gen();
            instruction = Instruction::decode(i);
            program = format!("{i:032b}");
            output = parse_machine_code(program);
            assert_eq!(Some(&instruction), output.first());
        }
    }

    #[test]
    fn loads_simple_program() {
        let simple_program = String::from(SIMPLE_PROGRAM);
        println!(";;{:?}", simple_program);
        let decoded = parse_machine_code(simple_program);
        println!(";;{:?}", decoded);
        assert_eq!(9, decoded.len());
        for i in decoded {
            println!("{}", i);
        }
        return ();
    }

    #[test]
    fn loads_simple_program_with_correct_instructions() {
        let simple_program = String::from(SIMPLE_PROGRAM);
        println!(";;{:?}", simple_program);
        let decoded = parse_machine_code(simple_program);
        println!(";;{:?}", decoded);
        assert_eq!(9, decoded.len());
        let mut i10 = Instruction::from_opcode(11);
        const TARGET:u8 = 1;
        i10.r_x_set(0);
        i10.i_y_set(10);
        i10.r_dest_set(TARGET);

        let mut i20 = Instruction::from_opcode(11);
        const VALUE:u8 = 2;
        i20.r_x_set(0);
        i20.i_y_set(0);
        i20.r_dest_set(VALUE);

        //put 11 in [3] for MAX
        let mut i30 = Instruction::from_opcode(11);
        const MAX:u8 = 3;
        i30.r_x_set(0);
        i30.i_y_set(11);
        i30.r_dest_set(MAX);

        //:Loop
        // put VALUE in (TARGET)
        let mut i35 = Instruction::from_opcode(23);
        i35.r_target_set(TARGET);
        i35.r_base_set(VALUE);
        i35.i_offset_set(0);

        //ADD 1 to VALUE
        let mut i40 = Instruction::from_opcode(12);
        i40.r_x_set(VALUE);
        i40.i_y_set(1);
        i40.r_dest_set(VALUE);

        //ADD 1 to TARGET
        let mut i50 = Instruction::from_opcode(12);
        i50.r_x_set(TARGET);
        i50.i_y_set(1);
        i50.r_dest_set(TARGET);

        // SUB VALUE from MAX
        let mut i60 = Instruction::from_opcode(13);
        i60.r_x_set(MAX);
        i60.r_y_set(VALUE);
        i60.r_dest_set(0);
        //Jump if GREATER than zero to Loop
        let mut i70 = Instruction::from_opcode(31);
        i70.i_set(12);
        i70.flags.greater = true;

        let i80 = Instruction::from_opcode(32);

        let all_instructions = vec!(
                                    i10,
                                    i20,
                                    i30,
                                    i35,
                                    i40,
                                    i50,
                                    i60,
                                    i70,
                                    i80,
        );
        for (i, j) in all_instructions.iter().zip(decoded) {
            assert_eq!(*i, j, "\n{}\n{}", i, j);
        }

/*
# 
0001-111111-00000 00011 00010 0001100 # 28 - 31
# End
0000-100000-00000 00000 00000 0000000 # 32 - 35
*/
        return ();
    }


/// Testing Programs
    const SIMPLE_PROGRAM:&'static str = "# Put numbers 0-10 into memory 10-20
# [#] Register
# (#) Memory 


#flg-opcode-00000 00000 00000 0000000
# put 10 in [1] as TARGET
0000-001011-00001 00000 00000 0001010 # 0-3
# put 0 in [2] as VALUE
0000-001011-00010 00000 00000 0000000 # 4-7
# put 11 in [3] for MAX
0000-001011-00011 00000 00000 0001011 # 8-11

:Loop
# put VALUE in (TARGET)
0000-010111-00001 00010 00000 0000000 # 12 - 15
# ADD 1 to VALUE
0000-001100-00010 00010 00000 0000001 # 16 - 19
# ADD 1 to TAGET
0000-001100-00001 00001 00000 0000001 # 20 - 23
# SUB VALUE from MAX
0000-001101-00000 00011 00010 0000000 # 24 - 27
# Jump if greater than Zero to Loop
0001-011111-00000 00000 00000 0001100 # 28 - 31
# End
0000-100000-00000 00000 00000 0000000 # 32 - 35
";
}
