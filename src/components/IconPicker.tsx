import React, { useState, useMemo, useRef, useEffect, useCallback } from 'react';
import { Search, X, ChevronRight } from 'lucide-react';
import { cn } from '../lib/utils';
import {
  ICON_CATEGORIES,
  getIconComponent,
  searchIcons,
  isValidIconName,
  DEFAULT_AGENT_ICON,
} from '../lib/iconRegistry';

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface IconPickerProps {
  /** Currently selected icon name */
  value: string | null;
  /** Callback when an icon is selected */
  onChange: (iconName: string) => void;
  /** Whether to show color selection (future use) */
  showColorPicker?: boolean;
  /** Selected color */
  color?: string;
  /** Callback when color changes */
  onColorChange?: (color: string) => void;
  /** CSS class for the trigger element */
  className?: string;
}

// ---------------------------------------------------------------------------
// IconPicker Component
// ---------------------------------------------------------------------------

export const IconPicker: React.FC<IconPickerProps> = ({
  value,
  onChange,
  showColorPicker = false,
  color,
  onColorChange,
  className,
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [activeCategory, setActiveCategory] = useState<string | null>(null);
  const popoverRef = useRef<HTMLDivElement>(null);
  const triggerRef = useRef<HTMLButtonElement>(null);

  // Close popover on outside click
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (
        popoverRef.current &&
        !popoverRef.current.contains(e.target as Node) &&
        triggerRef.current &&
        !triggerRef.current.contains(e.target as Node)
      ) {
        setIsOpen(false);
        setSearchQuery('');
        setActiveCategory(null);
      }
    };
    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    }
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [isOpen]);

  // Filtered icons
  const filteredIcons = useMemo(() => {
    let icons = searchIcons(searchQuery);
    if (activeCategory) {
      icons = icons.filter(name => {
        const cat = ICON_CATEGORIES.find(c => c.id === activeCategory);
        return cat ? cat.icons.includes(name) : true;
      });
    }
    return icons;
  }, [searchQuery, activeCategory]);

  const SelectedIcon = value && isValidIconName(value) ? getIconComponent(value) : null;

  const handleSelect = useCallback((iconName: string) => {
    onChange(iconName);
    setIsOpen(false);
    setSearchQuery('');
    setActiveCategory(null);
  }, [onChange]);

  return (
    <div className="relative">
      {/* Trigger Button */}
      <button
        ref={triggerRef}
        type="button"
        onClick={() => setIsOpen(!isOpen)}
        className={cn(
          "w-full brutal-border p-3 flex items-center gap-3 hover:bg-gray-50 transition-colors",
          className
        )}
      >
        <div className="w-10 h-10 brutal-border bg-brutal-cyan flex items-center justify-center shrink-0">
          {SelectedIcon ? (
            <SelectedIcon size={24} />
          ) : (
            <span className="text-lg">?</span>
          )}
        </div>
        <div className="flex-1 text-left">
          <div className="text-sm font-bold">{value || DEFAULT_AGENT_ICON}</div>
          <div className="text-[10px] text-gray-400">Click to change icon</div>
        </div>
        <ChevronRight size={16} className="text-gray-400" />
      </button>

      {/* Popover */}
      {isOpen && (
        <div
          ref={popoverRef}
          className="absolute z-50 left-0 top-full mt-2 w-80 brutal-border bg-white brutal-shadow"
        >
          {/* Search */}
          <div className="p-2 brutal-border-b">
            <div className="flex items-center gap-2 brutal-border bg-brutal-bg px-2 py-1">
              <Search size={14} className="text-gray-400 shrink-0" />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search icons..."
                className="flex-1 text-xs font-bold bg-transparent outline-none placeholder:text-gray-400"
                autoFocus
              />
              {searchQuery && (
                <button
                  onClick={() => setSearchQuery('')}
                  className="p-0.5 hover:bg-gray-200"
                >
                  <X size={12} />
                </button>
              )}
            </div>
          </div>

          {/* Category Tabs */}
          <div className="flex flex-wrap gap-1 p-2 brutal-border-b">
            <button
              onClick={() => setActiveCategory(null)}
              className={cn(
                "px-2 py-0.5 text-[9px] font-black uppercase brutal-border transition-all",
                !activeCategory ? "bg-brutal-yellow" : "bg-white hover:bg-gray-100"
              )}
            >
              All
            </button>
            {ICON_CATEGORIES.map((cat) => (
              <button
                key={cat.id}
                onClick={() => setActiveCategory(activeCategory === cat.id ? null : cat.id)}
                className={cn(
                  "px-2 py-0.5 text-[9px] font-black uppercase brutal-border transition-all",
                  activeCategory === cat.id ? "bg-brutal-yellow" : "bg-white hover:bg-gray-100"
                )}
              >
                {cat.label}
              </button>
            ))}
          </div>

          {/* Icon Grid */}
          <div className="p-2 max-h-60 overflow-y-auto">
            {filteredIcons.length === 0 ? (
              <div className="text-center text-xs text-gray-400 py-6 italic">
                No icons found for "{searchQuery}"
              </div>
            ) : (
              <div className="grid grid-cols-8 gap-1">
                {filteredIcons.map((iconName) => {
                  const Icon = getIconComponent(iconName);
                  if (!Icon) return null;
                  const isSelected = iconName === value;
                  return (
                    <button
                      key={iconName}
                      type="button"
                      onClick={() => handleSelect(iconName)}
                      title={iconName}
                      className={cn(
                        "w-8 h-8 flex items-center justify-center brutal-border transition-all hover:bg-brutal-bg",
                        isSelected
                          ? "bg-brutal-pink text-white brutal-shadow-sm translate-x-[-1px] translate-y-[-1px]"
                          : "bg-white hover:translate-x-[-1px] hover:translate-y-[-1px]"
                      )}
                    >
                      <Icon size={16} />
                    </button>
                  );
                })}
              </div>
            )}
          </div>

          {/* Preview / Selected */}
          {value && (
            <div className="p-2 brutal-border-t bg-gray-50 flex items-center gap-3">
              <div className="w-10 h-10 brutal-border bg-brutal-cyan flex items-center justify-center">
                {SelectedIcon ? <SelectedIcon size={24} /> : <span>?</span>}
              </div>
              <div>
                <div className="text-xs font-bold">{value}</div>
                <div className="text-[9px] text-gray-400">Selected icon</div>
              </div>
            </div>
          )}

          {/* Optional color picker (future expansion) */}
          {showColorPicker && onColorChange && (
            <div className="p-2 brutal-border-t">
              <div className="text-[9px] font-black uppercase text-gray-500 mb-1">Color</div>
              <div className="flex gap-1">
                {['bg-brutal-cyan', 'bg-brutal-pink', 'bg-brutal-yellow', 'bg-purple-400', 'bg-brutal-green', 'bg-orange-400', 'bg-teal-400', 'bg-red-400'].map((c) => (
                  <button
                    key={c}
                    type="button"
                    onClick={() => onColorChange(c)}
                    className={cn(
                      "w-6 h-6 brutal-border",
                      c,
                      color === c && "brutal-shadow-sm"
                    )}
                  />
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
