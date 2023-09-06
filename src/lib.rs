use std::rc::Rc;

use sqlite_loadable::prelude::*;
use sqlite_loadable::window::define_window_function_with_aux;
use sqlite_loadable::{api, Result};

use flatbuffers::FlatBufferBuilder;
use lean_buffer::{
    macros::LeanBufferWrite,
    traits::AdapterExt,
};

use base64::{Engine as _, engine::general_purpose};

// Don't panic when you see this false positive warning:
// proc macro `LeanBufferWrite` not expanded: proc macro not found in the built dylib
// Just check if the generated file can be located.
#[derive(LeanBufferWrite)]
pub struct Entity {
    x: String,
    y: i64,
}

// Either copy this file from your project, or use the name convention
// `<struct name>_lb_gen.rs` to include the generated file.
include!(concat!(env!("OUT_DIR"), "/Entity_lb_gen.rs"));

pub fn x_step(_context: *mut sqlite3_context, values: &[*mut sqlite3_value], aux: &mut Vec<Rc<Entity>>) -> Result<()> {
    let string_value = api::value_text(values.get(0).expect("should be text 1"));
    let int_value = api::value_int64(values.get(1).expect("should be int64"));

    aux.push(Rc::new(Entity { x: string_value.expect("should be text 2").to_string(), y: int_value }));

    Ok(())
}

pub fn x_final(context: *mut sqlite3_context, aux: &mut Vec<Rc<Entity>>) -> Result<()> {
    let mut builder = FlatBufferBuilder::new();
    
    let mut vec_entity_base64 = Vec::<String>::new();
    aux.iter().for_each(|t| {
        let entity = t.clone() as Rc<dyn AdapterExt>;
        entity.flatten(&mut builder);
        let buffer = builder.finished_data();
        vec_entity_base64.push(general_purpose::STANDARD_NO_PAD.encode(buffer));
    });

    api::result_text(context, vec_entity_base64.join("\n"))?;

    Ok(())
}

#[sqlite_entrypoint]
pub fn sqlite3_flatsqliters_init(db: *mut sqlite3) -> Result<()> {
    let flags = FunctionFlags::UTF8 | FunctionFlags::DETERMINISTIC;

    define_window_function_with_aux(
        db, "flat_string_int", 2, flags,
        x_step, x_final, None, None,
        Vec::<Rc::<Entity>>::new()
    )?;
    Ok(())
}

