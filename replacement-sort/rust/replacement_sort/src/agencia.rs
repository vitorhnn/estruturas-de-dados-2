use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::marker::Sized;
use std::string::FromUtf8Error;
use std::cmp::Ordering;
use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};


pub trait Serializable where Self: Sized {
    fn serialize(&self, file: &mut File) -> Result<(), SerializeError>;

    fn deserialize(file: &mut File) -> Result<Self, SerializeError>;
}

#[derive(Debug)]
pub struct Agencia {
    codigo: u32,
    nome: String, // encoded as 50 bytes of utf-8 in the files
    codigo_gerente: u32,
}

#[derive(Debug)]
pub enum SerializeError {
    Io(io::Error),
    Utf8(FromUtf8Error),
}

impl From<io::Error> for SerializeError {
    fn from(err: io::Error) -> SerializeError {
        SerializeError::Io(err)
    }
}

impl From<FromUtf8Error> for SerializeError {
    fn from(err: FromUtf8Error) -> SerializeError {
        SerializeError::Utf8(err)
    }
}

impl PartialEq for Agencia {
    fn eq(&self, other: &Agencia) -> bool {
        self.codigo == other.codigo
    }
}

impl PartialOrd for Agencia {
    fn partial_cmp(&self, other: &Agencia) -> Option<Ordering> {
        Some(self.codigo.cmp(&other.codigo))
    }
}

impl Serializable for Agencia {
    fn serialize(&self, file: &mut File) -> Result<(), SerializeError> {
        file.write_u32::<BigEndian>(self.codigo)?;

        let bytes = self.nome.as_bytes();
        let boundary = if bytes.len() > 50 { 50 } else { bytes.len() };
        let diff = 50 - boundary;
        file.write_all(&bytes[..boundary])?;

        for _ in 0..diff {
            file.write_u8(0)?;
        }

        file.write_u32::<BigEndian>(self.codigo_gerente)?;

        Ok(())
    }

    fn deserialize(file: &mut File) -> Result<Agencia, SerializeError> {
        let codigo = file.read_u32::<BigEndian>()?;
        let mut nome = [0; 50];
        file.read_exact(&mut nome)?;

        let nul_pos = nome.iter().position(|&c| c == b'\0').unwrap_or(nome.len());

        let nome = String::from_utf8(nome[..nul_pos].to_vec())?;

        let codigo_gerente = file.read_u32::<BigEndian>()?;

        Ok(Agencia {codigo, nome, codigo_gerente})
    }
}

