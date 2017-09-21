use std::cmp::{Ordering, min};
use std::fs::File;
use std::io::{BufWriter, Write, Read};

use chrono::{DateTime, NaiveDateTime, Utc};

use serializable::Serializable;
use serializable::SerializeError;

use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian};

#[derive(Debug, Clone)]
pub struct Cliente {
    pub codigo: u32,
    pub nome: String, // 50 bytes of utf-8 in the file
    pub data_nascimento: DateTime<Utc>,
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
        let mut file = BufWriter::new(file);

        if self.codigo == 3763629520 {
            println!("entra aqui arrombado");
        }

        file.write_u32::<BigEndian>(self.codigo)?;

        let bytes = self.nome.as_bytes();
        
        let boundary = min(50, bytes.len());
        let diff = 50 - boundary;
        file.write_all(&bytes[..boundary])?;

        for _ in 0..diff {
            file.write_u8(0)?;
        }

        file.write_i64::<BigEndian>(self.data_nascimento.timestamp())?;

        Ok(())
    }

    fn deserialize(file: &mut File) -> Result<Cliente, SerializeError> {
        let codigo = file.read_u32::<BigEndian>()?;
        let mut nome = [0; 50];
        file.read_exact(&mut nome)?;

        let nul_pos = nome.iter().position(|&c| c == b'\0').unwrap_or(nome.len());

        let nome = String::from_utf8(nome[..nul_pos].to_vec())?;

        let data_nascimento = file.read_i64::<BigEndian>()?;

        let data_nascimento = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(data_nascimento, 0), Utc);

        Ok(Cliente {codigo, nome, data_nascimento})
    }
}

