import fitz
import sys

PDF = sys.argv[1] if len(sys.argv) > 1 else "pdfs_origem/26.06.28.fatec.pdf"
PAGINA = int(sys.argv[2]) if len(sys.argv) > 2 else 14

doc = fitz.open(PDF)
p = doc[PAGINA - 1]

print(f"=== {PDF} — página {PAGINA} ===")
print(f"Tamanho: {p.rect.width} x {p.rect.height}")

texto = p.get_text()
print(f"\n--- get_text() ({len(texto)} chars) ---")
print(repr(texto[:800]))

print(f"\n--- get_text('blocks') ---")
for i, b in enumerate(p.get_text("blocks")):
    print(f"  bloco {i}: x0={b[0]:.0f} y0={b[1]:.0f} texto={b[4]!r}")

print(f"\n--- Imagens ({len(p.get_images(full=True))}) ---")
for img in p.get_images(full=True):
    print(f"  xref={img[0]} {img[2]}x{img[3]}")
