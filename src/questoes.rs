use crate::models::Questao;

pub async fn carregar_prova(caminho: &str) -> Vec<Questao> {
    let url = format!("/provas/{caminho}/questoes.json");

    let resp = gloo_net::http::Request::get(&url)
        .send()
        .await
        .unwrap_or_else(|e| panic!("Falha ao baixar {url}: {e}"));

    let mut questoes: Vec<Questao> = resp
        .json::<Vec<Questao>>()
        .await
        .unwrap_or_else(|e| panic!("JSON inválido em {url}: {e}"));

    for q in &mut questoes {
        for img in &mut q.imagens {
            let limpo = img.strip_prefix("assets/").unwrap_or(img);
            let arquivo = limpo.strip_prefix("imagens/").unwrap_or(limpo);
            *img = format!("/provas/{caminho}/imagens/{arquivo}");
        }
    }

    questoes
}
