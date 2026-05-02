export class OfficialSourceNormalizer {
  static normalize(source: string): string {
    return GanttTodayMarkerPolicy.normalize(source);
  }
}

class GanttTodayMarkerPolicy {
  static normalize(source: string): string {
    return GanttTodayMarkerPolicy.shouldAddOffMarker(source)
      ? `${source}\ntodayMarker off`
      : source;
  }

  private static shouldAddOffMarker(source: string): boolean {
    return GanttTodayMarkerPolicy.isGantt(source) && !GanttTodayMarkerPolicy.hasTodayMarker(source);
  }

  private static isGantt(source: string): boolean {
    return source.trimStart().startsWith("gantt");
  }

  private static hasTodayMarker(source: string): boolean {
    return /\btodayMarker\b/.test(source);
  }
}
