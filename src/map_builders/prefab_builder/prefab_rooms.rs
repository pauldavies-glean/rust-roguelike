#[derive(PartialEq, Copy, Clone)]
pub struct PrefabRoom {
    pub name: &'static str,
    pub template: &'static str,
    pub width: usize,
    pub height: usize,
    pub first_depth: i32,
    pub last_depth: i32,
}

pub const TOTALLY_NOT_A_TRAP: PrefabRoom = PrefabRoom {
    name: "totally not a trap",
    template: TOTALLY_NOT_A_TRAP_MAP,
    width: 5,
    height: 5,
    first_depth: 0,
    last_depth: 100,
};

const TOTALLY_NOT_A_TRAP_MAP: &str = "
     
 ^^^ 
 ^!^ 
 ^^^ 
     
";

pub const SILLY_SMILE: PrefabRoom = PrefabRoom {
    name: "silly smile",
    template: SILLY_SMILE_MAP,
    width: 6,
    height: 6,
    first_depth: 0,
    last_depth: 100,
};

const SILLY_SMILE_MAP: &str = "
      
 ^  ^ 
  #   
      
 ###  
      
";

pub const CHECKERBOARD: PrefabRoom = PrefabRoom {
    name: "checkerboard",
    template: CHECKERBOARD_MAP,
    width: 6,
    height: 5,
    first_depth: 0,
    last_depth: 100,
};

const CHECKERBOARD_MAP: &str = "
      
 g#%# 
 #!#  
 ^# # 
      
";
