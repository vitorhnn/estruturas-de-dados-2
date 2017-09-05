use std::cmp::Ordering;
use std::fs::File;
use std::io::Read;

use chrono::{NaiveDate};

use serializable::Serializable;
use serializable::SerializeError;

use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};

#[derive(Debug)]
pub struct Cliente {
    codigo: u32,
    nome: String, // 50 bytes of utf-8 in the file
    data_nascimento: NaiveDate, 
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

impl Serializable for Cliente {
    fn serialize(&self, file: &mut File) -> Result<(), SerializeError> {
        Ok(())
    }

    fn deserialize(file: &mut File) -> Result<Cliente, SerializeError> {
        let codigo = file.read_u32::<BigEndian>()?;
        let mut nome = [0; 50];
        file.read_exact(&mut nome)?;

        let nul_pos = nome.iter().position(|&c| c == b'\0').unwrap_or(nome.len());

        let nome = String::from_utf8(nome[..nul_pos].to_vec())?;

        let data_nascimento = file.read_i64::<BigEndian>()?;

        let data_nascimento = NaiveDate::parse_from_str(&data_nascimento.to_string(), "%s")?;

        Ok(Cliente {codigo, nome, data_nascimento})
    }
}

