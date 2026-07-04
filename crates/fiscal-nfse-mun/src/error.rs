//! Erros do crate de NFS-e municipal.

use std::fmt;

#[derive(Debug)]
pub enum MunError {
    /// Município não suportado (sem provedor registrado).
    MunicipioNaoSuportado(String),
    /// Operação ainda não implementada para o provedor.
    NaoImplementado(&'static str),
    /// Dado de entrada inválido.
    Validacao(String),
    /// Falha ao montar/serializar XML.
    Xml(String),
    /// Falha de assinatura.
    Assinatura(String),
    /// Falha de transporte (rede/SOAP/REST).
    Transporte(String),
}

impl fmt::Display for MunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MunError::MunicipioNaoSuportado(m) => write!(f, "município não suportado: {m}"),
            MunError::NaoImplementado(o) => write!(f, "não implementado: {o}"),
            MunError::Validacao(m) => write!(f, "validação: {m}"),
            MunError::Xml(m) => write!(f, "xml: {m}"),
            MunError::Assinatura(m) => write!(f, "assinatura: {m}"),
            MunError::Transporte(m) => write!(f, "transporte: {m}"),
        }
    }
}

impl std::error::Error for MunError {}

pub type Result<T> = std::result::Result<T, MunError>;
