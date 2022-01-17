use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::{Context, Entity, Event, LensWrap};

pub trait Model: 'static + Sized {
    fn build(self, cx: &mut Context) {
        if let Some(data_list) = cx.data.get_mut(cx.current) {
            // This might be a bad idea
            // if let Some(_) = data_list.get(&TypeId::of::<Self>()) {
            //     return;
            // }
            data_list.data.insert(TypeId::of::<Self>(), Box::new(self));
        } else {
            let mut data_list: HashMap<TypeId, Box<dyn ModelData>> = HashMap::new();
            data_list.insert(TypeId::of::<Self>(), Box::new(self));
            cx.data
                .insert(cx.current, ModelDataStore { data: data_list, lenses: HashMap::default() })
                .expect("Failed to add data");
        }
    }

    #[allow(unused_variables)]
    fn event(&mut self, cx: &mut Context, event: &mut Event) {}

    #[allow(unused_variables)]
    fn update(&mut self, cx: &mut Context) {}
}

pub trait ModelData: Any {
    #[allow(unused_variables)]
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        println!("Default");
    }

    fn update(&self) -> Vec<Entity> {
        println!("Default");
        Vec::new()
    }

    fn is_dirty(&self) -> bool {
        false
    }

    fn reset(&mut self) {}

    #[allow(unused_variables)]
    fn remove_observer(&mut self, observer: Entity) {}
}

impl dyn ModelData {
    // Check if a message is a certain type
    pub fn is<T: Any + 'static>(&self) -> bool {
        // Get TypeId of the type this function is instantiated with
        let t = TypeId::of::<T>();

        // Get TypeId of the type in the trait object
        let concrete = self.type_id();

        // Compare both TypeIds on equality
        t == concrete
    }

    // Casts a message to the specified type if the message is of that type
    pub fn downcast<T>(&mut self) -> Option<&mut T>
    where
        T: ModelData + 'static,
    {
        if self.is::<T>() {
            unsafe { Some(&mut *(self as *mut dyn ModelData as *mut T)) }
        } else {
            None
        }
    }

    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: Any + 'static,
    {
        if self.is::<T>() {
            unsafe { Some(&*(self as *const dyn ModelData as *const T)) }
        } else {
            None
        }
    }
}

trait Downcast {
    fn as_any(self: &'_ Self) -> &'_ dyn Any
    where
        Self: 'static;
}

impl<T: ModelData> Downcast for T {
    fn as_any(self: &'_ Self) -> &'_ dyn Any
    where
        Self: 'static,
    {
        self
    }
}

impl<T: Model> ModelData for T {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        <T as Model>::event(self, cx, event);
    }
}

#[derive(Default)]
pub struct ModelDataStore {
    pub data: HashMap<TypeId, Box<dyn ModelData>>,
    pub lenses: HashMap<TypeId, Box<dyn LensWrap>>,
}

impl Model for () {}
