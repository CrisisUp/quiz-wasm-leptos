//! Testes de integridade do `assets/questoes.json`.
//!
//! Roda no host (não em Wasm), então `std::fs` funciona normalmente.
//! Objetivo: falhar cedo se o `tools/ingest.py` gerar um JSON inválido,
//! com índices fora do range ou apontando para imagens inexistentes.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Questao {
    id: usize,
    pergunta: String,
    #[serde(default)]
    imagem: Option<String>,
    alternativas: Vec<String>,
    resposta_correta: usize,
}

const JSON_PATH: &str = "assets/provas/fatec_2026/questoes.json";
const IMAGENS_DIR: &str = "assets/provas/fatec_2026/imagens";

fn carregar() -> Vec<Questao> {
    let raw = fs::read_to_string(JSON_PATH)
        .unwrap_or_else(|e| panic!("Não foi possível ler {JSON_PATH}: {e}"));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("JSON inválido em {JSON_PATH}: {e}"))
}

#[test]
fn json_existe_e_e_um_array_nao_vazio() {
    assert!(
        Path::new(JSON_PATH).exists(),
        "{JSON_PATH} não existe — rode `make ingest` antes."
    );
    assert!(!carregar().is_empty(), "O JSON não tem nenhuma questão.");
}

#[test]
fn ids_sao_unicos() {
    let mut vistos = HashSet::new();
    for q in &carregar() {
        assert!(
            vistos.insert(q.id),
            "id duplicado: {} (questão: {})",
            q.id,
            q.pergunta
        );
    }
}

#[test]
fn pergunta_nao_e_vazia() {
    for q in &carregar() {
        assert!(
            !q.pergunta.trim().is_empty(),
            "questão id={} tem pergunta vazia",
            q.id
        );
    }
}

#[test]
fn tem_pelo_menos_duas_alternativas() {
    for q in &carregar() {
        assert!(
            q.alternativas.len() >= 2,
            "questão id={} tem {} alternativa(s); mínimo é 2",
            q.id,
            q.alternativas.len()
        );
    }
}

#[test]
fn alternativas_nao_sao_vazias() {
    for q in &carregar() {
        for (i, alt) in q.alternativas.iter().enumerate() {
            assert!(
                !alt.trim().is_empty(),
                "questão id={} tem alternativa {} vazia",
                q.id,
                i
            );
        }
    }
}

#[test]
fn resposta_correta_dentro_do_range() {
    for q in &carregar() {
        assert!(
            q.resposta_correta < q.alternativas.len(),
            "questão id={}: resposta_correta={} mas só há {} alternativa(s) (índices válidos: 0..{})",
            q.id,
            q.resposta_correta,
            q.alternativas.len(),
            q.alternativas.len() - 1
        );
    }
}

#[test]
fn imagens_referenciadas_existem_em_disco() {
    let mut faltando = Vec::new();
    for q in &carregar() {
        let Some(ref img) = q.imagem else { continue };
        let relativo = img.strip_prefix("assets/").unwrap_or(img);
        let caminho = Path::new("assets").join(relativo);
        if !caminho.exists() {
            faltando.push(format!(
                "  - questão id={}: {} (procurado em {})",
                q.id,
                img,
                caminho.display()
            ));
        }
    }
    assert!(
        faltando.is_empty(),
        "Imagens referenciadas no JSON não foram encontradas:\n{}",
        faltando.join("\n")
    );
}

#[test]
fn pasta_de_imagens_nao_tem_arquivos_orfaos() {
    let Ok(entries) = fs::read_dir(IMAGENS_DIR) else {
        eprintln!("Aviso: pasta {IMAGENS_DIR} não encontrada.");
        return;
    };

    let usadas: HashSet<String> = carregar()
        .iter()
        .filter_map(|q| q.imagem.as_ref())
        .map(|s| {
            let rel = s.strip_prefix("assets/").unwrap_or(s);
            rel.rsplit('/').next().unwrap_or(rel).to_string()
        })
        .collect();

    let mut orfas = Vec::new();
    for entry in entries.flatten() {
        if let Some(nome) = entry.file_name().to_str() {
            if nome.ends_with(".png") && !usadas.contains(nome) {
                orfas.push(nome.to_string());
            }
        }
    }
    if !orfas.is_empty() {
        eprintln!(
            "Aviso: {} imagem(ns) em disco não referenciada(s) no JSON:\n  {}",
            orfas.len(),
            orfas.join("\n  ")
        );
    }
}
