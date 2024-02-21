use std::collections::HashMap;
use std::fs;
use std::env;

#[allow(overflowing_literals)]
fn main() {
    let mut args = env::args();
    args.next();
    let flag = args.next().unwrap_or(String::from("failed"));
    let mut file_path = String::from("");
    match flag.as_str() {
        "-f" => {
            match args.next() {
                Some(s) => {
                    file_path = s;
                },
                None => {
                    println!("-f <filename> , -h");
                    return
                }
            }
        },
        _ => {println!("-f <filename> , -h"); 
        return
        }
    }
    let file_path = file_path.as_str();
    if file_path == "" {
        return
    }

    let inst_space = InstSpace::parse(file_path);
    let mut vm = Vm::new(inst_space);

    let mut running = true;

    while running{
        running = vm.exucute();    
    }

    println!("{:?}", vm.regs);
    println!("{:?}", vm.mem)
}

struct Vm {
    inst_space: InstSpace,
    regs: [i32; 32],
    pc: u32,
    mem: HashMap<u32, i32>
}
impl Vm {
    fn new(is : InstSpace) -> Vm{
        Vm {
            inst_space: is,
            regs: [0i32; 32],
            pc: 0,
            mem: HashMap::new()
        }
    }
    fn exucute(&mut self) -> bool {
        if self.pc / 4 >= self.inst_space.prog.len().try_into().unwrap()  {
           false 
        } else {
            let word = self.inst_space.word_at(self.pc).clone();
            let instruction = construct_inst(word);
            instruction.exucute(self);
            self.pc += 4;
            true
        }
    }
}

struct InstSpace{
    prog: Vec<u32>
}

impl InstSpace {
    fn new(v: Vec<u32>) -> InstSpace {
        InstSpace{
            prog: v
        }
    }

    fn word_at(&self, i: u32) -> u32 {
        self.prog[(i / 4) as usize]
    }

    fn parse(s: &str) -> InstSpace {
        let file = fs::read_to_string(s).expect("Read File");
        let file_iter = file.split("\n");

        let mut program_contents:Vec<u32> = vec![];
        for line in file_iter {
            program_contents.push(u32::from_str_radix(line, 16).unwrap());
        }
        InstSpace::new(program_contents)

    }
}

fn get_bits(inst: u32, start: u8, length: u8 ) -> u32{
    ((inst << 32 - (length + start)) >> 32 - length) as u32
}

fn get_inst_seg(inst: u32, seg_type: String) -> u32 {
    let range_dict = HashMap::from([
        (String::from("op"), (26,6)),
        (String::from("rs"), (21,5)),
        (String::from("rt"), (16,5)),
        (String::from("rd"), (11,5)),
        (String::from("sh"), (6,5)),
        (String::from("fn"), (0,6)),
        (String::from("im"), (0,16)),
        (String::from("ad"), (0,26)),
    ]);
    let range = range_dict.get(&seg_type).copied();
    match range {
        Some(x) => get_bits(inst, x.0, x.1),
        None => panic!(),
    }
}

fn construct_inst(inst: u32) -> Box<dyn Inst>{
    match get_inst_seg(inst, String::from("op")){
        0 => {
            Box::new( RType {
                op : get_inst_seg(inst, String::from("op")),
                rs : get_inst_seg(inst, String::from("rs")),
                rt : get_inst_seg(inst, String::from("rt")),
                rd : get_inst_seg(inst, String::from("rd")),
                sh : get_inst_seg(inst, String::from("sh")),
                fun : get_inst_seg(inst, String::from("fn")),
            })
        },
        2 | 3 => {
            Box::new( JType {
                op : get_inst_seg(inst, String::from("op")),
                ad : get_inst_seg(inst, String::from("ad")),
            })
        },
        0x4 | 0x8 | 0x9 | 0xC | 0x5 | 0x23 | 0xD | 0xA | 0x2B=> 
        {
            Box::new( IType {
                op : get_inst_seg(inst, String::from("op")),
                rs : get_inst_seg(inst, String::from("rs")),
                rt : get_inst_seg(inst, String::from("rt")),
                im : get_inst_seg(inst, String::from("im")),
            })

        },
        _ => panic!(),

    }
}

pub trait Inst {
    fn exucute(&self, vm: &mut Vm );
}
pub struct RType {
    op: u32,
    rs: u32,
    rt: u32,
    rd: u32,
    sh: u32,
    fun: u32,
}

impl Inst for RType {
    fn exucute(&self, vm: &mut Vm) {
        match self.fun {
            // ADD
            0x20 => {
                vm.regs[self.rd as usize] = vm.regs[self.rs as usize] + vm.regs[self.rt as usize];
            },
            // ADDU
            0x21 => {
                vm.regs[self.rd as usize] = vm.regs[self.rs as usize] + vm.regs[self.rt as usize];
            },
            // AND
            0x24 =>{
                vm.regs[self.rd as usize] = vm.regs[self.rs as usize] & vm.regs[self.rt as usize];
            },
            // OR
            0x25 => {
                vm.regs[self.rd as usize] = vm.regs[self.rs as usize] | vm.regs[self.rt as usize];
            },
            // NOR
            0x27 => {
                vm.regs[self.rd as usize] = !(vm.regs[self.rs as usize] | vm.regs[self.rt as usize]);
            },
            // SLL
            0x00 => {
                vm.regs[self.rd as usize] = vm.regs[self.rt as usize] << self.sh;
            },
            // SRL
            0x02 => {
                vm.regs[self.rd as usize] = vm.regs[self.rt as usize] >> self.sh;
            },
            // SUB
            0x22 => {
                vm.regs[self.rd as usize] = vm.regs[self.rs as usize] - vm.regs[self.rt as usize];
            },
            // SUBU
            0x23 => {
                vm.regs[self.rd as usize] = vm.regs[self.rs as usize] - vm.regs[self.rt as usize];
            }
            // JR
            0x08 => {
                vm.pc = vm.regs[self.rs as usize] as u32 - 4;
            },
            // SLT
            0x2A => {
                if vm.regs[self.rs as usize] < vm.regs[self.rt as usize] {
                    vm.regs[self.rd as usize] = 1; 
                } else {
                    vm.regs[self.rd as usize] = 0; 

                }
            },
            _ => panic!()
        }
    } 
}

struct IType {
    op: u32,
    rs: u32,
    rt: u32,
    im: u32,
}

impl Inst for IType {
    fn exucute(&self,vm: &mut Vm) {
        match self.op {
            // ADDI
            0x8 => {
                vm.regs[self.rt as usize] = vm.regs[self.rs as usize] + read_imm(self.im);
            },
            // ADDIU
            0x9 => {
                vm.regs[self.rt as usize] = vm.regs[self.rs as usize] + read_imm(self.im);
            },
            // ANDI
            0xC => {
                vm.regs[self.rt as usize] = (self.im & vm.regs[self.rs as usize] as u32) as i32;
            },
            // BEQ
            0x4 => {
                if (vm.regs[self.rt as usize] == vm.regs[self.rs as usize]) {vm.pc += 4 + self.im}
            },
            // BNE
            0x5 => {
                if vm.regs[self.rt as usize] != vm.regs[self.rs as usize] {vm.pc += 4 + self.im}
            },
            // ORI
            0xD => {
                vm.regs[self.rt as usize] = (self.im | vm.regs[self.rs as usize] as u32) as i32;
            },
            // SLTI
            0xA => {
                if vm.regs[self.rs as usize] < read_imm(self.im) {
                    vm.regs[self.rt as usize] = 1;
                } else {
                    vm.regs[self.rt as usize] = 0;
                }
            },
            // SW
            0x2B => {
                vm.mem.insert((vm.regs[self.rs as usize]+read_imm(self.im)) as u32, vm.regs[self.rt as usize]);
            },
            // LW
            0x23 => {
                let loc = (vm.regs[self.rs as usize]+read_imm(self.im)) as u32;
                match vm.mem.get(&loc) {
                    Some(inner) => vm.regs[self.rt as usize] = *inner,
                    None => panic!("Tried to Access unwriten memory"),
                }
            }
            
            _ => panic!()
        }
    }
}

struct JType {
    op: u32,
    ad: u32,
}

impl Inst for JType {
    fn exucute(&self, vm: &mut Vm) {
        match self.op {
            // J
            0x2 => {
                vm.pc = self.ad;
            },
            // JAL
            0x3 => {
                vm.regs[31] = (vm.pc + 4) as i32;
                vm.pc = self.ad;
            }
            _ => panic!()
        }
    }
}

fn read_imm(x: u32) -> i32{
    if x << 16 >> 31 == 0{
        return x as i32
    }
    let a = (x | 0xFFFF0000) ^ 0xFFFF_FFFF;
    if a == 0xFFFF_FFFF {
       0 
    } else {
        -1 *(a+1) as i32
    }
}