use std::cmp::{Ordering, min};
use std::fs::File;
use std::io::{Write, Read};

use serialize::Serialize;
use serialize::SerializeError;

use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};

#[derive(Debug, Clone)]
pub struct Cliente {
    pub codigo: u32,
    pub nome: String, // 100 bytes of utf-8 in the file
}

impl PartialEq for Cliente {
    fn eq(&self, other: &Cliente) -> bool {
        self.codigo == other.codigo
    }
}

impl PartialOrd for Cliente {
    fn partial_cmp(&self, other: &Cliente) -> Option<Ordering> {
        Some(self.codigo.cmp(&other.codigo))
    }
}

impl Serialize for Cliente {
    fn serialize(&self, file: &mut File) -> Result<(), SerializeError> {
        file.write_u32::<BigEndian>(self.codigo)?;

        let bytes = self.nome.as_bytes();
        
        let boundary = min(100, bytes.len());
        let diff = 100 - boundary;
        file.write_all(&bytes[..boundary])?;

        for _ in 0..diff {
            file.write_u8(0)?;
        }

        Ok(())
    }

    fn deserialize(file: &mut File) -> Result<Cliente, SerializeError> {
        let codigo = file.read_u32::<BigEndian>()?;
        let mut nome = [0; 100];
        file.read_exact(&mut nome)?;

        let nul_pos = nome.iter().position(|&c| c == b'\0').unwrap_or(nome.len());

        let nome = String::from_utf8(nome[..nul_pos].to_vec())?;

        Ok(Cliente {codigo, nome})
    }
}

