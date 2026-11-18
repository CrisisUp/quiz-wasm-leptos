use crate::models::{Questao, Resultado};
use std::collections::HashMap;

#[derive(Debug)]
pub struct QuizEngine {
    pub questoes: Vec<Questao>,
    pub indice: usize,
    pub acertos: usize,
    pub respostas: Vec<Option<usize>>,
    pub placar_por_materia: HashMap<String, (usize, usize)>,
}

impl QuizEngine {
    pub fn new(questoes: Vec<Questao>) -> Self {
        let n = questoes.len();
        let mut placar = HashMap::new();
        for q in &questoes {
            let materia = q.materia.clone().unwrap_or_else(|| "Geral".to_string());
            let entry = placar.entry(materia).or_insert((0, 0));
            entry.1 += 1;
        }

        Self {
            questoes,
            indice: 0,
            acertos: 0,
            respostas: vec![None; n],
            placar_por_materia: placar,
        }
    }

    pub fn total(&self) -> usize {
        self.questoes.len()
    }
    pub fn finalizado(&self) -> bool {
        self.indice >= self.questoes.len()
    }

    pub fn responder(&mut self, escolhida: usize) -> Option<Resultado> {
        if self.finalizado() {
            return None;
        }
        if self.respostas[self.indice].is_some() {
            return None;
        }

        let q = self.questoes.get(self.indice)?;
        if escolhida >= q.alternativas.len() {
            return None;
        }

        let correta = escolhida == q.resposta_correta;

        if correta {
            self.acertos += 1;
            let materia = q.materia.clone().unwrap_or_else(|| "Geral".to_string());
            if let Some((acertos_mat, _)) = self.placar_por_materia.get_mut(&materia) {
                *acertos_mat += 1;
            }
        }

        self.respostas[self.indice] = Some(escolhida);
        Some(Resultado {
            correta,
            indice_correto: q.resposta_correta,
            explicacao: q.explicacao.clone(),
        })
    }

    pub fn avancar(&mut self) -> bool {
        if self.indice < self.questoes.len() {
            self.indice += 1;
        }
        !self.finalizado()
    }

    pub fn porcentagem(&self) -> f64 {
        if self.questoes.is_empty() {
            return 0.0;
        }
        (self.acertos as f64 / self.questoes.len() as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(id: usize, correta: usize, materia: &str) -> Questao {
        Questao {
            id,
            pergunta: format!("Pergunta {id}"),
            imagens: vec![],
            alternativas: vec!["A".into(), "B".into(), "C".into(), "D".into()],
            resposta_correta: correta,
            explicacao: None,
            materia: Some(materia.to_string()),
        }
    }

    #[test]
    fn placar_por_materia_funciona() {
        let mut e = QuizEngine::new(vec![
            q(1, 0, "Biologia"),
            q(2, 1, "Biologia"),
            q(3, 2, "Matemática"),
        ]);
        e.responder(0);
        e.avancar();
        e.responder(0);
        e.avancar();
        e.responder(2);
        assert_eq!(*e.placar_por_materia.get("Biologia").unwrap(), (1, 2));
        assert_eq!(*e.placar_por_materia.get("Matemática").unwrap(), (1, 1));
    }

    #[test]
    fn materia_geral_para_questoes_sem_materia() {
        let mut e = QuizEngine::new(vec![Questao {
            id: 1,
            pergunta: "?".into(),
            imagens: vec![],
            alternativas: vec!["A".into(), "B".into()],
            resposta_correta: 0,
            explicacao: None,
            materia: None,
        }]);
        e.responder(0);
        assert_eq!(*e.placar_por_materia.get("Geral").unwrap(), (1, 1));
    }
}
