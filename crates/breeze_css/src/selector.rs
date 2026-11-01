#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Combinator {
    Descendant,
    Child,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimpleSelector {
    Universal,
    Tag(String),
    Id(String),
    Class(String),
    Attribute {
        name: String,
        value: Option<String>,
    },
    PseudoClass(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundSelector {
    pub simple_selectors: Vec<SimpleSelector>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComplexSelector {
    pub compound: CompoundSelector,
    pub next: Option<(Combinator, Box<ComplexSelector>)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Specificity {
    pub a: u32,
    pub b: u32,
    pub c: u32,
    pub d: u32,
}

impl Specificity {
    pub fn new(a: u32, b: u32, c: u32, d: u32) -> Self {
        Self { a, b, c, d }
    }
}

impl ComplexSelector {
    pub fn specificity(&self) -> Specificity {
        let mut b = 0;
        let mut c = 0;
        let mut d = 0;

        let mut curr = Some(self);
        while let Some(sel) = curr {
            for simple in &sel.compound.simple_selectors {
                match simple {
                    SimpleSelector::Id(_) => b += 1,
                    SimpleSelector::Class(_)
                    | SimpleSelector::Attribute { .. }
                    | SimpleSelector::PseudoClass(_) => c += 1,
                    SimpleSelector::Tag(_) => d += 1,
                    SimpleSelector::Universal => {}
                }
            }
            curr = sel.next.as_ref().map(|(_, next_sel)| next_sel.as_ref());
        }

        Specificity::new(0, b, c, d)
    }
}
