type LogContext = Record<string, unknown>;

interface RendererErrorPayload {
  code?: string;
  message: string;
  context?: LogContext;
  stack?: string;
  source?: string;
}

function extractStack(error: unknown): string | undefined {
  if (!error) return undefined;
  if (error instanceof Error) return error.stack || error.message;
  if (typeof error === 'string') return error;
  try {
    return JSON.stringify(error);
  } catch {
    return String(error);
  }
}

export function reportError(code: string, message: string, error?: unknown, context?: LogContext): void {
  const stack = extractStack(error);

  if (process.env.NODE_ENV !== 'production') {
    // 开发环境仍然在控制台输出，方便调试
    // eslint-disable-next-line no-console
    console.error(message, error, context);
  }

  const payload: RendererErrorPayload = {
    code,
    message,
    context,
    stack,
    source: 'renderer',
  };

  try {
    window.electronAPI?.reportError?.(payload).catch(() => {
      // 上报失败时静默忽略，避免影响业务流程
    });
  } catch {
    // ignore
  }
}

