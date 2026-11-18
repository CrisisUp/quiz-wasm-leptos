#!/usr/bin/env python3
"""
Recorta figuras (inclusive vetoriais) do PDF e salva em assets/imagens.
Uso: python3 tools/recortar_figuras.py <numero_da_questao>
Exemplo: python3 tools/recortar_figuras.py 46
"""

import sys
import pymupdf as fitz
from pathlib import Path

RAIZ = Path(__file__).resolve().parent.parent
PDF_PATH = RAIZ / "pdfs_origem" / "26.06.28.fatec.pdf"
DIR_IMAGENS = RAIZ / "assets" / "imagens"

# Coordenadas aproximadas (x0, y0, x1, y1) no PDF para cada questão que tem figura complexa.
# Você pode ajustar esses valores abrindo o PDF no Preview e vendo as propriedades (Cmd+I).
COORDENADAS = {
    # Ex: Q46 (página 17) - A bola Trionda no topo
    # O PyMuPDF usa a origem (0,0) no canto superior esquerdo da página.
    46: {"pagina": 17, "box": (150, 100, 450, 300)}, 
    # Ex: Q10 (página 5) - O gráfico de radar
    10: {"pagina": 5, "box": (100, 150, 400, 400)},
}

def recortar(numero_questao: int):
    if numero_questao not in COORDENADAS:
        print(f"Nenhuma coordenada mapeada para a Q{numero_questao}.")
        return

    info = COORDENADAS[numero_questao]
    num_pagina = info["pagina"] - 1  # 0-indexado no PyMuPDF
    box = fitz.Rect(info["box"]) # (x0, y0, x1, y1)

    doc = fitz.open(PDF_PATH)
    pagina = doc[num_pagina]

    # Renderiza a área recortada como uma imagem em alta resolução
    pix = pagina.get_pixmap(matrix=fitz.Matrix(3, 3), clip=box)
    
    nome = f"q{numero_questao}_figura.png"
    caminho = DIR_IMAGENS / nome
    pix.save(caminho)

    print(f"✓ Figura da Q{numero_questao} salva em {caminho}")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Uso: python3 tools/recortar_figuras.py <numero_da_questao>")
        sys.exit(1)
    
    recortar(int(sys.argv[1]))