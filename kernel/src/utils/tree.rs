use alloc::vec::Vec;

pub struct Tree<Item> {
    pub value: Item,
    pub children: Vec<Tree<Item>>,
}

impl<Item> Tree<Item> {
    fn new(value: Item) -> Tree<Item> {
        Tree {
            value,
            children: Vec::new(),
        }
    }

    pub fn root(value: Item) -> Tree<Item> {
        Tree::new(value)
    }

    pub fn add_child(&mut self, child: Item) {
        self.children.push(Tree::new(child));
    }
}
