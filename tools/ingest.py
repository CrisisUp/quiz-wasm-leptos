#!/usr/bin/env python3
"""
Ingest do PDF da FATEC (prova + gabarito).

REGRA DE OURO:
  - Se `assets/questoes_editado.json` existir, o script NÃO toca nele.
    Ele assume que o usuário fez edições manuais e gera só o "bruto"
    em `questoes_bruto.json` (para comparação).
  - Se `questoes_editado.json` não existir, salva em `questoes.json`.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

try:
    import pymupdf as fitz
except ImportError:  # pragma: no cover
    import fitz  # type: ignore


# --- Caminhos ---------------------------------------------------------------
RAIZ = Path(__file__).resolve().parent.parent
PDF_PADRAO = RAIZ / "pdfs_origem" / "26.06.28.fatec.pdf"
PDF_GABARITO = RAIZ / "pdfs_origem" / "26.06.28.fatec-gabarito.pdf"
DIR_IMAGENS = RAIZ / "assets" / "imagens"

ARQ_EDITADO = RAIZ / "assets" / "questoes_editado.json"  # versão manual (prioridade)
ARQ_BRUTO = RAIZ / "assets" / "questoes_bruto.json"       # versão gerada (debug)
ARQ_JSON = RAIZ / "assets" / "questoes.json"             # alvo padrão


# --- Regex ------------------------------------------------------------------
RE_CABECALHO = re.compile(r"Quest[ãa]o\s*(\d+)", re.IGNORECASE)
RE_ALTERNATIVA = re.compile(r"^\s*\(?([A-Ea-e])\)[\.\s]?\s*(.+?)\s*$", re.MULTILINE)
RE_GABARITO = re.compile(r"\b(\d{3})\s+([A-E])\s+([A-Za-zÀ-ÖØ-öø-ÿ\s]+)")

RE_CORTE_TEXTO_BASE = re.compile(
    r"(Leia o texto|Leia a charge|Leia o trecho|Analise a figura|Observe a imagem|Considere o texto|Com base no texto|O texto a seguir|TEXTO PARA AS PRÓXIMAS QUESTÕES)",
    re.IGNORECASE,
)

RE_RODAPE = re.compile(
    r"(\d+\s*VESTIBULAR|VESTIBULAR\s*\d+|Reservados todos os direitos|Reprodução proibida)",
    re.IGNORECASE
)

RE_LINK_VAZADO = re.compile(r"<https?://[^>]+>\s*Acesso em:\s*\d{2}\.\d{2}\.\d{4}\.", re.IGNORECASE)

LETRA_PARA_INDICE = {"A": 0, "B": 1, "C": 2, "D": 3, "E": 4}


# --- Utilitários ------------------------------------------------------------
def normalizar(texto: str) -> str:
    texto = texto.replace("\xa0", " ")
    texto = re.sub(r"-\n(?=\w)", "", texto)
    texto = re.sub(r"[ \t]+", " ", texto)
    return texto.strip()


# --- Leitura do gabarito ----------------------------------------------------
def ler_gabarito(caminho: Path) -> dict[int, tuple[int, str]]:
    """Retorna {numero_da_questao: (indice_0_a_4, materia)}."""
    if not caminho.exists():
        print(f"AVISO: gabarito não encontrado em {caminho}", file=sys.stderr)
        return {}

    doc = fitz.open(caminho)
    texto = "\n".join(p.get_text() for p in doc)
    respostas: dict[int, tuple[int, str]] = {}
    
    for m in RE_GABARITO.finditer(texto):
        num = int(m.group(1))
        letra = m.group(2)
        materia = m.group(3).strip()
        if num not in respostas:
            respostas[num] = (LETRA_PARA_INDICE[letra], materia)
    return respostas


# --- Extração das questões da prova ----------------------------------------
class Bloco:
    __slots__ = ("y0", "y1", "texto")

    def __init__(self, y0: float, y1: float, texto: str):
        self.y0 = y0
        self.y1 = y1
        self.texto = texto


def coletar_blocos(doc):
    paginas = []
    for n_pag, pagina in enumerate(doc, start=1):
        altura = pagina.rect.height
        limite_rodape = altura * 0.92
        blocos = []
        for b in pagina.get_text("blocks"):
            if b[6] != 0:
                continue
            y0, y1, texto = b[1], b[3], b[4]
            if y0 > limite_rodape and "Quest" not in texto:
                continue
            if texto.strip():
                blocos.append(Bloco(y0, y1, texto))
        blocos.sort(key=lambda b: b.y0)
        paginas.append((n_pag, blocos))
    return paginas


def separar_questoes(paginas):
    for n_pag, blocos in paginas:
        cabecalhos = []
        for b in blocos:
            m = RE_CABECALHO.search(b.texto)
            if m:
                cabecalhos.append((b.y0, int(m.group(1))))

        for i, (y_cab, num) in enumerate(cabecalhos):
            y_fim = cabecalhos[i + 1][0] if i + 1 < len(cabecalhos) else float("inf")
            partes = [b.texto for b in blocos if y_cab <= b.y0 < y_fim]
            texto = "\n".join(partes)
            texto = RE_CABECALHO.sub("", texto, count=1)
            texto = normalizar(texto)
            m_corte = RE_CORTE_TEXTO_BASE.search(texto)
            if m_corte and m_corte.start() > 50:
                texto = texto[:m_corte.start()]
            texto = RE_RODAPE.sub("", texto)
            texto = RE_LINK_VAZADO.sub("", texto)
            texto = normalizar(texto)
            if texto:
                yield num, n_pag, texto, (y_cab, y_fim)


def extrair_imagens(pagina, y_intervalo, numero: int, n_pag: int) -> list[Path]:
    y_ini, y_fim = y_intervalo
    salvas: list[Path] = []
    try:
        infos = pagina.get_image_info(xrefs=True)
    except Exception:
        return salvas
    idx = 0
    for info in infos:
        bbox = info.get("bbox")
        xref = info.get("xref", 0)
        if not bbox or not xref:
            continue
        if not (y_ini <= bbox[1] < y_fim):
            continue
        try:
            base = pagina.parent.extract_image(xref)
        except Exception:
            continue
        idx += 1
        ext = base.get("ext", "png")
        nome = f"pag{n_pag}_q{numero}_img{idx}.{ext}"
        (DIR_IMAGENS / nome).write_bytes(base["image"])
        salvas.append(Path("imagens") / nome)
    return salvas


def parsear_alternativas(texto: str) -> tuple[str, list[str]]:
    linhas = texto.split("\n")
    if len(linhas) > 1 and linhas[0].strip() and linhas[0].strip() in linhas[1]:
        texto = "\n".join(linhas[1:])

    matches = list(RE_ALTERNATIVA.finditer(texto))
    if len(matches) < 2:
        return texto.strip(), []

    inicio = matches[0].start()
    enunciado = texto[:inicio].strip()
    enunciado = re.sub(r"\s+\d+\s+\d+$", "", enunciado).strip()

    alternativas: list[str] = []
    for i, m in enumerate(matches):
        letra = m.group(1).upper()
        corpo = re.sub(r"^[\.\s]+", "", m.group(2).strip())
        fim = matches[i + 1].start() if i + 1 < len(matches) else len(texto)
        resto = texto[m.end():fim].strip()
        resto = re.sub(r"^\d+\s*\n?\d+\s*\n?", "", resto)
        resto = re.sub(r"[\r\n]+", " ", resto)
        resto = re.sub(r"\s*\([A-Ea-e]\)\s*$", "", resto).strip()
        resto = RE_RODAPE.sub("", resto).strip()
        resto = RE_LINK_VAZADO.sub("", resto).strip()
        conteudo = f"{letra}) {corpo} {resto}".strip()
        if conteudo and conteudo != f"{letra})":
            alternativas.append(conteudo)

    if not (2 <= len(alternativas) <= 5):
        return texto.strip(), []
    return enunciado, alternativas


def montar_questao(numero, n_pag, texto, imagens, gabarito) -> dict:
    enunciado, alternativas = parsear_alternativas(texto)

    if alternativas:
        curtas = sum(1 for alt in alternativas if len(alt) < 6)
        if curtas >= 2 or len(alternativas) < 5:
            alternativas = []

    if not alternativas:
        alternativas = [f"{l}) ..." for l in "ABCDE"]

    while len(alternativas) < 5:
        idx = len(alternativas)
        alternativas.append(f"{chr(65 + idx)}) ...")

    # Pega (resposta, materia) do gabarito
    resposta, materia = gabarito.get(numero, (0, "Geral"))

    return {
        "id": numero,
        "pergunta": enunciado or f"Questão {numero} (página {n_pag})",
        "imagens": [str(imagens[0])] if imagens else [],
        "alternativas": alternativas,
        "resposta_correta": resposta,
        "explicacao": None,
        "materia": materia,
    }


# --- Main -------------------------------------------------------------------
def main() -> int:
    prova = Path(sys.argv[1]) if len(sys.argv) > 1 else PDF_PADRAO
    gab_path = Path(sys.argv[2]) if len(sys.argv) > 2 else PDF_GABARITO

    if not prova.exists():
        print(f"ERRO: prova não encontrada: {prova}", file=sys.stderr)
        return 1

    gabarito = ler_gabarito(gab_path)
    print(f"Gabarito lido: {len(gabarito)} respostas")

    DIR_IMAGENS.mkdir(parents=True, exist_ok=True)
    for pat in ("*.png", "*.jpg", "*.jpeg"):
        for f in DIR_IMAGENS.glob(pat):
            f.unlink()

    print(f"Lendo prova: {prova}")
    doc = fitz.open(prova)
    paginas = coletar_blocos(doc)
    print(f"Páginas: {len(paginas)}")

    paginas_por_num = {n: p for n, p in enumerate(doc, start=1)}
    questoes = []
    for numero, n_pag, texto, y_int in separar_questoes(paginas):
        pagina = paginas_por_num[n_pag]
        imagens = extrair_imagens(pagina, y_int, numero, n_pag)
        questoes.append(montar_questao(numero, n_pag, texto, imagens, gabarito))

    questoes.sort(key=lambda q: q["id"])

    if ARQ_EDITADO.exists():
        saida = ARQ_BRUTO
        print(f"\n⚠️  {ARQ_EDITADO.name} existe.")
        print(f"   Salvando bruto em {saida.name} (o editado NÃO é tocado).")
        print(f"   Se quiser regenerar, apague {ARQ_EDITADO.name} primeiro.")
    else:
        saida = ARQ_JSON
        print(f"\nSalvando em {saida.name} (primeiro run).")

    saida.write_text(
        json.dumps(questoes, ensure_ascii=False, indent=2),
        encoding="utf-8",
    )
    print(f"✓ {len(questoes)} questões em {saida}")
    return 0


if __name__ == "__main__":
    sys.exit(main())