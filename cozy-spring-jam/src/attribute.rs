use std::{collections::HashMap, hash::Hash};

use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum Operation {
    Add(f32),
    Multiply(f32),
}

impl Operation {
    fn apply(&self, val: &mut f32) {
        match self {
            Self::Add(x) => *val += x,
            Self::Multiply(x) => *val *= x,
        }
    }

    fn order(&self) -> usize {
        match self {
            Self::Add(_) => 0,
            Self::Multiply(_) => 1,
        }
    }
}

#[derive(Debug, Clone)]
struct Modifier<A> {
    attribute: A,
    operation: Operation,
}

#[derive(Debug, Clone)]
pub struct Effect<A> {
    id: Uuid,
    modifiers: Vec<Modifier<A>>,
}

impl<A> Effect<A> {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            modifiers: Vec::new(),
        }
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }

    pub fn add_modifier(&mut self, attribute: A, operation: Operation) {
        self.modifiers.push(Modifier {
            attribute,
            operation,
        });
    }
}

pub struct Attributes<A> {
    base: HashMap<A, f32>,
    temporary_effects: HashMap<Uuid, Effect<A>>,
    cache: HashMap<A, f32>,
}

impl<A> Attributes<A> {
    pub fn new() -> Self {
        Self {
            base: HashMap::new(),
            temporary_effects: HashMap::new(),
            cache: HashMap::new(),
        }
    }
}

impl<A> Default for Attributes<A> {
    fn default() -> Self {
        Self::new()
    }
}

impl<A: Copy + Eq + Hash> Attributes<A> {
    pub fn set_base(&mut self, attr: A, value: f32) -> &mut Self {
        self.base.insert(attr, value);
        self
    }

    pub fn get_base(&self, attr: A) -> f32 {
        self.base.get(&attr).copied().unwrap_or_default()
    }

    pub fn apply_effect(&mut self, effect: Effect<A>) -> &mut Self {
        self.clear_cache_for_effect(&effect);
        self.temporary_effects.insert(effect.get_id(), effect);
        self
    }

    pub fn remove_effect(&mut self, id: Uuid) -> &mut Self {
        if let Some(effect) = self.temporary_effects.remove(&id) {
            self.clear_cache_for_effect(&effect);
        }
        self
    }

    pub fn get(&mut self, attr: A) -> f32 {
        if let Some(cached) = self.cache.get(&attr) {
            return *cached;
        }

        let mut value = self.get_base(attr);

        let mut operations: Vec<Operation> = Vec::new();
        for effect in self.temporary_effects.values() {
            for modifier in &effect.modifiers {
                if modifier.attribute == attr {
                    operations.push(modifier.operation.clone());
                }
            }
        }
        operations.sort_by_key(Operation::order);

        for operation in operations {
            operation.apply(&mut value);
        }

        self.cache.insert(attr, value);
        value
    }

    pub fn get_int(&mut self, attr: A) -> i32 {
        self.get(attr).round() as i32
    }

    pub fn get_uint(&mut self, attr: A) -> u32 {
        self.get(attr).round() as u32
    }

    fn clear_cache_for_effect(&mut self, effect: &Effect<A>) {
        for modifier in &effect.modifiers {
            self.cache.remove(&modifier.attribute);
        }
    }
}
