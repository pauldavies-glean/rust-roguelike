#[allow(dead_code)]
#[derive(PartialEq, Copy, Clone)]
pub enum HorizontalPlacement {
    Left,
    Center,
    Right,
}

#[allow(dead_code)]
#[derive(PartialEq, Copy, Clone)]
pub enum VerticalPlacement {
    Top,
    Center,
    Bottom,
}

#[derive(PartialEq, Copy, Clone)]
pub struct PrefabSection {
    pub name: &'static str,
    pub template: &'static str,
    pub width: usize,
    pub height: usize,
    pub placement: (HorizontalPlacement, VerticalPlacement),
}

pub const UNDERGROUND_FORT: PrefabSection = PrefabSection {
    name: "underground fort",
    template: RIGHT_FORT,
    width: 15,
    height: 43,
    placement: (HorizontalPlacement::Right, VerticalPlacement::Top),
};

// The padding needs to be here!
const RIGHT_FORT: &str = "
     #         
  #######      
  #     #      
  #     #######
  #  g        #
  #     #######
  #     #      
  ### ###      
    # #        
    # #        
    # ##       
    ^          
    ^          
    # ##       
    # #        
    # #        
    # #        
    # #        
  ### ###      
  #     #      
  #     #      
  #  g  #      
  #     #      
  #     #      
  ### ###      
    # #        
    # #        
    # #        
    # ##       
    ^          
    ^          
    # ##       
    # #        
    # #        
    # #        
  ### ###      
  #     #      
  #     #######
  #  g        #
  #     #######
  #     #      
  #######      
     #         
";
