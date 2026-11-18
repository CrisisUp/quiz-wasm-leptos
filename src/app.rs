use gloo_timers::future::TimeoutFuture;
use leptos::*;

use quiz_wasm_leptos::engine::QuizEngine;
use quiz_wasm_leptos::models::Resultado;
use quiz_wasm_leptos::questoes;
use quiz_wasm_leptos::storage;

const CHAVE_MELHOR: &str = "quiz.melhor_pontuacao";

const PROVAS_DISPONIVEIS: &[(&str, &str)] = &[
    ("fatec_2026", "FATEC 2026"),
    ("fatec_2025/semestre-2", "FATEC 2025 - 2º Semestre"),
    ("fatec_2024", "FATEC 2024"),
];

#[derive(Clone)]
struct EstadoResposta {
    escolhida: usize,
    correta: usize,
    acertou: bool,
    explicacao: Option<String>,
}

#[component]
pub fn App() -> impl IntoView {
    let (prova_selecionada, set_prova_selecionada) = create_signal::<Option<String>>(None);

    view! {
        <main style="display: flex; flex-direction: column; align-items: center; width: 100%;">
            <header class="header-cps">
                <div class="logo-placeholder">"CPS"</div>
                <h1>"Vestibular FATEC"</h1>
            </header>

            {move || match prova_selecionada.get() {
                None => view! {
                    <div class="quiz-card">
                        <h2>"Escolha uma prova para começar"</h2>
                        <p style="color: #666; margin-bottom: 1.5rem;">
                            "Selecione uma das provas abaixo:"
                        </p>
                        <div style="display: flex; flex-direction: column; gap: 12px;">
                            {PROVAS_DISPONIVEIS.iter().map(|(caminho, nome)| {
                                let caminho_str = caminho.to_string();
                                view! {
                                    <button
                                        class="opcao"
                                        on:click=move |_| set_prova_selecionada.set(Some(caminho_str.clone()))
                                    >
                                        {*nome}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    </div>
                }.into_view(),
                Some(caminho) => view! {
                    <QuizApp caminho=caminho />
                }.into_view(),
            }}
        </main>
    }
}

#[component]
fn QuizApp(caminho: String) -> impl IntoView {
    let questoes_resource = create_local_resource(
        move || caminho.clone(),
        |caminho| async move { questoes::carregar_prova(&caminho).await },
    );

    view! {
        {move || match questoes_resource.get() {
            None => view! {
                <div class="quiz-card">
                    <h2>"Carregando prova..."</h2>
                    <p style="color: #666;">"Buscando questões e imagens."</p>
                </div>
            }.into_view(),
            Some(questoes_vec) => view! {
                <QuizInterno questoes=questoes_vec />
            }.into_view(),
        }}
    }
}

#[component]
fn QuizInterno(questoes: Vec<quiz_wasm_leptos::models::Questao>) -> impl IntoView {
    let total = questoes.len();
    let engine = store_value(QuizEngine::new(questoes));

    let (indice, set_indice) = create_signal(0usize);
    let (acertos, set_acertos) = create_signal(0usize);
    let (resposta_atual, set_resposta_atual) = create_signal::<Option<EstadoResposta>>(None);
    let (finalizado, set_finalizado) = create_signal(false);
    let (zoom_src, set_zoom_src) = create_signal::<Option<String>>(None);

    let melhor_inicial = storage::carregar::<u32>(CHAVE_MELHOR);
    let (melhor, set_melhor) = create_signal::<Option<u32>>(melhor_inicial);

    let responder = move |escolhida: usize| {
        if resposta_atual.get_untracked().is_some() {
            return;
        }
        let resultado: Option<Resultado> = engine
            .try_update_value(|e| e.responder(escolhida))
            .flatten();
        let Some(r) = resultado else { return };

        if r.correta {
            set_acertos.update(|a| *a += 1);
        }
        set_resposta_atual.set(Some(EstadoResposta {
            escolhida,
            correta: r.indice_correto,
            acertou: r.correta,
            explicacao: r.explicacao,
        }));

        spawn_local(async move {
            TimeoutFuture::new(1200).await;
            set_resposta_atual.set(None);
            let ainda_tem = engine.try_update_value(|e| e.avancar()).unwrap_or(false);
            if ainda_tem {
                set_indice.update(|i| *i += 1);
            } else {
                set_finalizado.set(true);
                let pontos = acertos.get_untracked() as u32;
                let novo_melhor = melhor
                    .get_untracked()
                    .map(|m| m.max(pontos))
                    .unwrap_or(pontos);
                set_melhor.set(Some(novo_melhor));
                storage::salvar(CHAVE_MELHOR, &novo_melhor);
            }
        });
    };

    view! {
        <div class="quiz-card">
            {move || if finalizado.get() {
                let pontos = acertos.get();
                let pct = if total == 0 { 0.0 } else { (pontos as f64 / total as f64) * 100.0 };
                view! {
                    <div>
                        <h2>"Fim do Questionário!"</h2>
                        <p>"Você acertou " <strong>{pontos}</strong> " de " <strong>{total}</strong> " perguntas."</p>
                        <p>"Desempenho Geral: " <strong>{format!("{pct:.1}%")}</strong></p>

                        <h3 style="margin-top: 2rem;">"Desempenho por Matéria"</h3>
                        <table style="width: 100%; border-collapse: collapse; margin-top: 1rem;">
                            <thead>
                                <tr style="border-bottom: 2px solid var(--cps-red); text-align: left;">
                                    <th style="padding: 8px;">Matéria</th>
                                    <th style="padding: 8px; text-align: center;">Acertos</th>
                                    <th style="padding: 8px; text-align: center;">Total</th>
                                </tr>
                            </thead>
                            <tbody>
                                {engine.with_value(|e| {
                                    let mut materias: Vec<_> = e.placar_por_materia.iter().collect();
                                    materias.sort_by_key(|(nome, _)| nome.to_string());
                                    materias.into_iter().map(|(nome, (acertos, total_mat))| {
                                        view! {
                                            <tr style="border-bottom: 1px solid #eee;">
                                                <td style="padding: 8px;">{nome.clone()}</td>
                                                <td style="padding: 8px; text-align: center; font-weight: bold;">{*acertos}</td>
                                                <td style="padding: 8px; text-align: center; color: #666;">{*total_mat}</td>
                                            </tr>
                                        }
                                    }).collect_view()
                                })}
                            </tbody>
                        </table>

                        {melhor.get().map(|m| view! {
                            <p style="color:#666; font-size:0.9rem; margin-top: 1.5rem;">
                                "Melhor pontuação registrada: " <strong>{m}</strong>
                            </p>
                        })}

                        <button
                            class="btn-reiniciar"
                            on:click=move |_| {
                                if let Some(win) = web_sys::window() {
                                    let _ = win.location().reload();
                                }
                            }
                        >
                            "Reiniciar"
                        </button>
                    </div>
                }.into_view()
            } else {
                render_pergunta(engine, indice, resposta_atual, set_zoom_src, responder, total).into_view()
            }}
        </div>

        {move || zoom_src.get().map(|src| view! {
            <div class="modal-zoom" on:click=move |_| set_zoom_src.set(None)>
                <img src=src alt="Imagem ampliada"/>
            </div>
        })}
    }
}

fn render_pergunta<F>(
    engine: StoredValue<QuizEngine>,
    indice: ReadSignal<usize>,
    resposta_atual: ReadSignal<Option<EstadoResposta>>,
    set_zoom_src: WriteSignal<Option<String>>,
    responder: F,
    total: usize,
) -> impl IntoView
where
    F: Fn(usize) + Copy + 'static,
{
    let q = engine.with_value(|e| e.questoes[indice.get()].clone());
    let q_pergunta = q.pergunta.clone();
    let q_alternativas = q.alternativas.clone();
    let q_imagens = q.imagens.clone();

    view! {
        <div>
            <h3>"Questão " {indice.get() + 1} " de " {total}</h3>
            <p class="pergunta">{q_pergunta}</p>

            {if !q_imagens.is_empty() {
                view! {
                    <div class="galeria-imagens">
                        {q_imagens.into_iter().map(|src| {
                            let src_click = src.clone();
                            view! {
                                <img class="imagem-questao" src=src alt="Figura da questão"
                                     on:click=move |_| set_zoom_src.set(Some(src_click.clone())) />
                            }
                        }).collect_view()}
                    </div>
                }.into_view()
            } else { view! { <div></div> }.into_view() }}

            <div class="opcoes">
                {q_alternativas.into_iter().enumerate().map(|(i, alt)| {
                    let estado = resposta_atual.get();
                    let classe = match estado {
                        Some(ref e) if i == e.correta => "opcao correta",
                        Some(ref e) if i == e.escolhida => "opcao errada",
                        _ => "opcao",
                    };
                    let desabilitado = estado.is_some();
                    view! {
                        <button class=classe disabled=desabilitado on:click=move |_| responder(i) inner_html=alt />
                    }
                }).collect_view()}
            </div>

            <p class="feedback">
                {move || match resposta_atual.get() {
                    Some(e) if e.acertou => "Resposta Correta! 🚀".to_string(),
                    Some(_) => "Resposta Errada! ❌".to_string(),
                    None => String::new(),
                }}
            </p>

            {move || match resposta_atual.get() {
                Some(e) if !e.acertou => e.explicacao.map(|ex| view! { <p class="explicacao">{ex}</p> }),
                _ => None,
            }}
        </div>
    }
}
