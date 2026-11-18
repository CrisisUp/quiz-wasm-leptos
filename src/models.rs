use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Questao {
    pub id: usize,
    pub pergunta: String,
    #[serde(default)]
    pub imagens: Vec<String>,
    pub alternativas: Vec<String>,
    pub resposta_correta: usize,
    pub explicacao: Option<String>,
    #[serde(default)]
    pub materia: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resultado {
    pub correta: bool,
    pub indice_correto: usize,
    pub explicacao: Option<String>,
}
