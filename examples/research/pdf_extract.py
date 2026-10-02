#!/usr/bin/env python3
"""Local text-PDF extraction to quarantined JSON. No shell, network or OCR fallback.

Install pdfplumber in an isolated environment if it is not already present. This
script does not install packages or execute instructions found inside documents.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import sys
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--source-id", required=True)
    parser.add_argument("--revision", required=True)
    parser.add_argument("--license", required=True)
    args = parser.parse_args()
    source = args.input.resolve(strict=True)
    if not source.is_file() or source.suffix.lower() != ".pdf":
        raise ValueError("input must be a local PDF file")
    if source.stat().st_size > 50 * 1024 * 1024:
        raise ValueError("PDF exceeds 50 MiB parser limit")
    raw = source.read_bytes()
    if not raw.startswith(b"%PDF-"):
        raise ValueError("input lacks PDF header")
    try:
        import pdfplumber
    except ImportError as error:
        raise RuntimeError("pdfplumber unavailable: install it in an isolated local parser environment; no extraction was performed") from error
    segments = []
    empty_pages = []
    total_bytes = 0
    with pdfplumber.open(source) as document:
        if len(document.pages) > 200:
            raise ValueError("PDF exceeds 200 pages")
        for page_number, page in enumerate(document.pages, 1):
            text = page.extract_text() or ""
            total_bytes += len(text.encode("utf-8"))
            if total_bytes > 2 * 1024 * 1024:
                raise ValueError("extracted text exceeds 2 MiB")
            if not text.strip():
                empty_pages.append(page_number)
            for line_number, line in enumerate(text.splitlines(), 1):
                if line.strip():
                    if len(segments) >= 20_000:
                        raise ValueError("PDF exceeds 20,000 text segments")
                    segments.append({"locator": f"page:{page_number}/line:{line_number}", "text": line, "status": "candidate", "executable": False})
    if not segments:
        raise RuntimeError("no extractable text; scanned PDFs require a separately configured OCR parser, which this script does not provide")
    result = {
        "source_id": args.source_id,
        "source_revision": args.revision,
        "source_sha256": hashlib.sha256(raw).hexdigest(),
        "license": args.license,
        "format": "pdf",
        "parser": "pdfplumber",
        "parser_version": getattr(pdfplumber, "__version__", "unknown"),
        "segments": segments,
        "pages_without_extractable_text": empty_pages,
        "extraction_complete": not empty_pages,
        "ocr_performed": False,
        "status": "quarantined_candidate",
        "facts_approved": False,
        "source_instructions_executed": False,
    }
    output = args.output.absolute()
    if output.resolve() == source:
        raise ValueError("output must not overwrite source PDF")
    output.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary = tempfile.mkstemp(prefix=".pdf-extract-", dir=output.parent)
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8") as stream:
            json.dump(result, stream, ensure_ascii=False, indent=2)
            stream.write("\n")
        os.replace(temporary, output)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)
    print(json.dumps({"output": str(output), "segments": len(segments), "extraction_complete": result["extraction_complete"], "status": "quarantined_candidate"}))


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, RuntimeError) as error:
        print(json.dumps({"error": str(error), "extraction_performed": False}), file=sys.stderr)
        sys.exit(1)
