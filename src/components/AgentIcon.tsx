import React from 'react';
import { cn } from '../lib/utils';
import { getIconComponent, isValidIconName } from '../lib/iconRegistry';

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface AgentIconProps {
  /** SVG icon name from the icon registry, or null/undefined for emoji fallback */
  icon?: string | null;
  /** Emoji fallback when icon is not set */
  emoji?: string;
  /** Size preset: 'sm' (sidebar), 'md' (chat), 'lg' (profile) */
  size?: 'sm' | 'md' | 'lg';
  /** Background color class (e.g. 'bg-brutal-cyan') */
  bgColor?: string;
  /** Additional CSS classes */
  className?: string;
  /** Tooltip text */
  title?: string;
}

// ---------------------------------------------------------------------------
// Size map
// ---------------------------------------------------------------------------

const SIZE_MAP: Record<string, { container: string; icon: number; text: string }> = {
  sm: { container: 'w-6 h-6', icon: 12, text: 'text-sm' },
  md: { container: 'w-8 h-8', icon: 18, text: 'text-base' },
  lg: { container: 'w-10 h-10', icon: 24, text: 'text-xl' },
};

const LG_SIZE_MAP = {
  container: 'w-20 h-20',
  icon: 48,
  text: 'text-4xl',
};

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/**
 * Unified agent icon renderer.
 *
 * - If `icon` is a valid SVG icon name, renders the corresponding Lucide icon.
 * - Otherwise falls back to displaying `emoji` (first character).
 * - If neither is available, uses the default agent icon (Bot).
 */
export const AgentIcon: React.FC<AgentIconProps> = ({
  icon,
  emoji,
  size = 'md',
  bgColor = 'bg-brutal-cyan',
  className,
  title,
}) => {
  const hasValidIcon = icon && isValidIconName(icon);
  const IconComponent = hasValidIcon ? getIconComponent(icon!) : null;
  const isLarge = size === 'lg';

  const sizeConfig = isLarge ? LG_SIZE_MAP : SIZE_MAP[size];

  // When we have a valid SVG icon
  if (IconComponent) {
    return (
      <div
        title={title}
        className={cn(
          sizeConfig.container,
          'brutal-border flex items-center justify-center shrink-0',
          bgColor,
          className
        )}
      >
        <IconComponent size={sizeConfig.icon} />
      </div>
    );
  }

  // Fallback to emoji
  const displayChar = emoji?.charAt(0) || 'A';
  return (
    <div
      title={title}
      className={cn(
        sizeConfig.container,
        'brutal-border flex items-center justify-center shrink-0 font-black',
        sizeConfig.text,
        bgColor,
        className
      )}
    >
      {displayChar}
    </div>
  );
};
