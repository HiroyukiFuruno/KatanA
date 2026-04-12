import json
import os

locales_dir = "crates/katana-ui/locales"
languages = {
    "en": {"match_case": "Match Case", "match_word": "Match Word", "use_regex": "Use Regex", "doc_query_hint": "Search in document…", "filter_hint": "Filter…"},
    "ja": {"match_case": "大文字・小文字を区別", "match_word": "単語単位", "use_regex": "正規表現を利用", "doc_query_hint": "文書内を検索…", "filter_hint": "フィルター…"},
    "de": {"match_case": "Groß-/Kleinschreibung", "match_word": "Ganzes Wort", "use_regex": "Regulärer Ausdruck", "doc_query_hint": "Im Dokument suchen…", "filter_hint": "Filtern…"},
    "es": {"match_case": "Coincidir mayúsculas", "match_word": "Palabra completa", "use_regex": "Usar Regex", "doc_query_hint": "Buscar en el documento…", "filter_hint": "Filtrar…"},
    "fr": {"match_case": "Respecter la casse", "match_word": "Mot entier", "use_regex": "Utiliser Regex", "doc_query_hint": "Rechercher dans le document…", "filter_hint": "Filtrer…"},
    "it": {"match_case": "Case sensitive", "match_word": "Parola intera", "use_regex": "Usa Regex", "doc_query_hint": "Cerca nel documento…", "filter_hint": "Filtra…"},
    "pt": {"match_case": "Diferenciar maiúsculas", "match_word": "Palavra inteira", "use_regex": "Usar Regex.", "doc_query_hint": "Pesquisar no documento…", "filter_hint": "Filtragem…"},
    "ko": {"match_case": "대소문자 구분", "match_word": "단어 단위", "use_regex": "정규식 사용", "doc_query_hint": "문서 내 검색…", "filter_hint": "필터…"},
    "zh-CN": {"match_case": "区分大小写", "match_word": "全字匹配", "use_regex": "使用正则", "doc_query_hint": "在文档内搜索…", "filter_hint": "过滤…"},
    "zh-TW": {"match_case": "區分大小寫", "match_word": "全字比對", "use_regex": "使用正規表示式", "doc_query_hint": "在文件内搜尋…", "filter_hint": "篩選…"}
}

for lang, terms in languages.items():
    path = os.path.join(locales_dir, f"{lang}.json")
    if not os.path.exists(path):
        continue
    
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)
        
    if "search" not in data: data["search"] = {}
    data["search"]["match_case"] = terms["match_case"]
    data["search"]["match_word"] = terms["match_word"]
    data["search"]["use_regex"] = terms["use_regex"]
    data["search"]["doc_query_hint"] = terms["doc_query_hint"]
    
    if "workspace" not in data: data["workspace"] = {}
    data["workspace"]["filter_hint"] = terms["filter_hint"]

    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=4)
    print(f"Fixed {lang}.json")
