export function formatTimestamp(value: string, locale: string = 'zh-CN'): string {
  if (!value) return '-';
  const toDate = (s: string | number) => {
    const d = new Date(s);
    return isNaN(d.getTime()) ? null : d;
  };
  let d = toDate(value);
  if (!d) {
    const sqlPattern = /^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}(?:\.\d{1,3})?$/;
    if (sqlPattern.test(value)) {
      const candidate = value.replace(' ', 'T') + 'Z';
      d = toDate(candidate);
    }
  }
  if (!d && /^\d{10,13}$/.test(value)) {
    const ms = value.length === 13 ? Number(value) : Number(value) * 1000;
    d = toDate(ms);
  }
  if (!d) return value;
  const opts: Intl.DateTimeFormatOptions = {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false
  };
  return d.toLocaleString(locale, opts);
}
