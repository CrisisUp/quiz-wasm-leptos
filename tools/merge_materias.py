#!/usr/bin/env python3
import json
from pathlib import Path

RAIZ = Path(__file__).resolve().parent.parent
EDITADO = RAIZ / "assets/questoes_editado.json"
BRUTO = RAIZ / "assets/questoes_bruto.json"

if not EDITADO.exists():
    print(f"ERRO: {EDITADO.name} não existe.")
    exit(1)
if not BRUTO.exists():
    print(f"ERRO: {BRUTO.name} não existe. Rode o ingest.py primeiro.")
    exit(1)

editado = json.loads(EDITADO.read_text(encoding="utf-8"))
bruto = json.loads(BRUTO.read_text(encoding="utf-8"))

# Cria um dicionário {id: materia} baseado no bruto
materias = {q["id"]: q.get("materia", "Geral") for q in bruto}

# Injeta a matéria no editado
for q in editado:
    q["materia"] = materias.get(q["id"], "Geral")

EDITADO.write_text(json.dumps(editado, ensure_ascii=False, indent=2), encoding="utf-8")
print(f"✓ Matérias injetadas em {len(editado)} questões do {EDITADO.name}.")
