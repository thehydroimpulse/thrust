#![feature(plugin)]
#![plugin(thrust_macros)]

extern crate thrust;
extern crate tangle;

use thrust::binary_protocol::*;
use thrust::protocol::*;
use std::io::{Cursor, Read, Write};

thrust!("
    namespace rust foobar1

    service FlockDb {
        map<string, byte> query(1: string voodoo, 2: i32 mission_control);
    }
");

#[test]
fn args_deserialize_gen() {
    let mut buf = Vec::new();

    {
        let mut se = BinarySerializer::new(&mut buf);
        let args = foobar1::FlockDb_query_Args {
            voodoo: "Hello".to_string(),
            mission_control: 500
        };

        args.serialize(&mut se).unwrap();
    }

    let mut rd = Cursor::new(buf);
    let mut de = BinaryDeserializer::new(rd);
    let args = foobar1::FlockDb_query_Args::deserialize(&mut de).unwrap();
    assert_eq!(&*args.voodoo, "Hello");
    assert_eq!(args.mission_control, 500);
}

#[test]
fn manual_args_deserialize() {
    let mut buf = Vec::new();

    {
        let mut se = BinarySerializer::new(&mut buf);
        se.write_struct_begin("FlockDb_query_Args").unwrap();

        se.write_field_begin("voodoo", ThriftType::String, 1).unwrap();
        "Hello".to_string().serialize(&mut se);
        se.write_field_stop();
        se.write_field_end();

        se.write_field_begin("mission_control", ThriftType::I32, 2).unwrap();
        let i: i32 = 500;
        i.serialize(&mut se);
        se.write_field_stop();
        se.write_field_end();

        se.write_struct_end();
    }

    let mut rd = Cursor::new(buf);
    let mut de = BinaryDeserializer::new(rd);
    let msg = de.read_struct_begin().unwrap();
    let voodoo = de.read_field_begin().unwrap();
    assert!(voodoo.name.is_none());
    assert_eq!(voodoo.seq, 1);
    assert_eq!(voodoo.ty, ThriftType::String);
    assert_eq!(&*de.deserialize_str().unwrap(), "Hello");
    de.read_field_begin().unwrap();
    let mission = de.read_field_begin().unwrap();
    assert_eq!(mission.seq, 2);
    assert_eq!(mission.ty, ThriftType::I32);
    assert_eq!(de.deserialize_i32().unwrap(), 500);
}

#[test]
fn serialize() {
    let mut buf = Vec::new();
    let mut comp = Vec::new();

    {
        let mut se = BinarySerializer::new(&mut buf);
        let args = foobar1::FlockDb_query_Args {
            voodoo: "Hello".to_string(),
            mission_control: 500
        };

        args.serialize(&mut se).unwrap();
    }

    {
        let mut se = BinarySerializer::new(&mut comp);
        se.write_struct_begin("FlockDb_query_Args").unwrap();

        se.write_field_begin("voodoo", ThriftType::String, 1).unwrap();
        "Hello".to_string().serialize(&mut se);
        se.write_field_stop();
        se.write_field_end();

        se.write_field_begin("mission_control", ThriftType::I32, 2).unwrap();
        let i: i32 = 500;
        i.serialize(&mut se);
        se.write_field_stop();
        se.write_field_end();

        se.write_struct_end();
    }

    assert_eq!(buf.len(), comp.len());

    for i in 0..buf.len() {
        assert_eq!(buf[i], comp[i]);
    }
}
