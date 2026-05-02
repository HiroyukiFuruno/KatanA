function katanaDrawioIsPaintServer(value) {
  return String(value).trim().toLowerCase().startsWith("url(");
}
