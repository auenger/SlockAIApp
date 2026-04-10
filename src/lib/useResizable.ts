import { useState, useCallback, useRef, useEffect } from 'react';

interface UseResizableOptions {
  /** Initial width in pixels */
  initialWidth: number;
  /** Minimum width in pixels */
  minWidth: number;
  /** Maximum width in pixels */
  maxWidth: number;
  /** Which edge the handle is on: 'right' means drag right to increase, 'left' means drag left to increase */
  edge: 'right' | 'left';
}

interface UseResizableReturn {
  /** Current width in pixels */
  width: number;
  /** Whether a resize drag is in progress */
  isResizing: boolean;
  /** Ref to attach to the resize handle element */
  handleRef: React.RefObject<HTMLDivElement | null>;
  /** Style object to spread on the resizable container */
  style: React.CSSProperties;
}

/**
 * Hook for panel width resizing via drag handle.
 *
 * - `edge: 'right'` — handle on the right edge, dragging right increases width (e.g. Sidebar)
 * - `edge: 'left'`  — handle on the left edge, dragging left increases width (e.g. ThreadPanel)
 */
export function useResizable({ initialWidth, minWidth, maxWidth, edge }: UseResizableOptions): UseResizableReturn {
  const [width, setWidth] = useState(initialWidth);
  const [isResizing, setIsResizing] = useState(false);
  const handleRef = useRef<HTMLDivElement | null>(null);
  const startXRef = useRef(0);
  const startWidthRef = useRef(0);

  const clamp = useCallback((w: number) => Math.min(maxWidth, Math.max(minWidth, w)), [minWidth, maxWidth]);

  const onMouseDown = useCallback((e: MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);
    startXRef.current = e.clientX;
    startWidthRef.current = width;
  }, [width]);

  const onMouseMove = useCallback((e: MouseEvent) => {
    const delta = edge === 'right'
      ? e.clientX - startXRef.current
      : startXRef.current - e.clientX;
    const next = clamp(startWidthRef.current + delta);
    setWidth(next);
  }, [edge, clamp]);

  const onMouseUp = useCallback(() => {
    setIsResizing(false);
  }, []);

  // Attach mousedown to handle element
  useEffect(() => {
    const el = handleRef.current;
    if (!el) return;
    el.addEventListener('mousedown', onMouseDown);
    return () => {
      el.removeEventListener('mousedown', onMouseDown);
    };
  }, [onMouseDown]);

  // Global mousemove / mouseup during drag
  useEffect(() => {
    if (!isResizing) return;
    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
    // Prevent text selection while dragging
    document.body.style.userSelect = 'none';
    document.body.style.cursor = 'col-resize';
    return () => {
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
      document.body.style.userSelect = '';
      document.body.style.cursor = '';
    };
  }, [isResizing, onMouseMove, onMouseUp]);

  return {
    width,
    isResizing,
    handleRef,
    style: { width: `${width}px`, flexShrink: 0 },
  };
}
