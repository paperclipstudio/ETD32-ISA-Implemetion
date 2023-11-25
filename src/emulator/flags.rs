use std::fmt;

#[derive(PartialEq)]
pub struct Flags {
    pub carry: bool,
    pub greater: bool,
    pub zero: bool,
    pub less: bool,
}

impl Flags {
    pub fn new() -> Flags {
        Flags {
            carry: false,
            greater: false,
            zero: false,
            less: false,
        }
    }

    pub fn set_all_flags(&mut self, state:bool) {
            self.carry = state;
            self.less = state;
            self.zero = state;
            self.greater = state;
    }

    pub fn instruction_can_run(cpu_flags: &Flags, instruction_flags: &Flags) -> bool {
        // If the instruction flag is set and the cpu flag isn't then return false
        // OR
        // if the instruction flag isn't set or the cpu flag is the return true
        //  I  C   Out
        //  0  0   1
        //  0  1   1
        //  1  0   0
        //  1  1   1
            (!instruction_flags.zero || cpu_flags.zero) && 
            (!instruction_flags.less || cpu_flags.less) &&
            (!instruction_flags.greater || cpu_flags.greater) &&
            (!instruction_flags.carry || cpu_flags.carry)
    }
}

impl fmt::Display for Flags {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", if self.carry {"C"} else {"-"})?;
        write!(fmt, "{}", if self.greater {"G"} else {"-"})?;
        write!(fmt, "{}", if self.zero {"Z"} else {"-"})?;
        write!(fmt, "{}", if self.less {"L"} else {"-"})?;
        Ok(())
    }
}

impl fmt::Debug for Flags {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "Carry: {}", self.carry)?;
        writeln!(fmt, "Greater: {}", self.greater)?;
        writeln!(fmt, "Zero: {}", self.zero)?;
        writeln!(fmt, "Less: {}", self.less)?;
        Ok(())
    }
}


#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn instruction_can_run_default_true() {
        let cpu:Flags = Flags::new();
        let inst:Flags = Flags::new();
        assert!(Flags::instruction_can_run(&cpu, &inst));
    }

    #[test]
    fn instruction_can_run_zero_block() {
        let cpu:Flags = Flags::new();
        let mut inst:Flags = Flags::new();
        inst.zero = true;
        assert!(!Flags::instruction_can_run(&cpu, &inst));
    }

    #[test]
    fn instruction_can_run_cpu_zero_does_not_block() {
        let mut cpu:Flags = Flags::new();
        cpu.zero = true;
        let inst:Flags = Flags::new();
        assert!(Flags::instruction_can_run(&cpu, &inst));
    }

    #[test]
    fn instruction_can_run_less_block() {
        let cpu:Flags = Flags::new();
        let mut inst:Flags = Flags::new();
        inst.less = true;
        assert!(!Flags::instruction_can_run(&cpu, &inst));
    }

    #[test]
    fn instruction_can_run_cpu_less_does_not_block() {
        let mut cpu:Flags = Flags::new();
        cpu.less = true;
        let inst:Flags = Flags::new();
        assert!(Flags::instruction_can_run(&cpu, &inst));
    }

}
