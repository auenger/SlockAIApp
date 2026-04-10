/**
 * SVG Icon Registry for the AgentsZone application.
 *
 * Curates a subset of lucide-react icons organized by category,
 * providing a unified icon selection system for Agents and Users.
 */

import {
  // Characters / People
  Bot,
  User,
  Users,
  UserCircle,
  Ghost,
  Skull,
  Crown,
  Sparkles,
  Star,
  Heart,
  Shield,
  Swords,

  // Animals / Nature
  Bird,
  Bug,
  Cat,
  Dog,
  Fish,
  Leaf,
  TreePine,
  Flame,
  Snowflake,
  Sun,
  Moon,
  Cloud,
  Rainbow,
  Mountain,
  Waves,

  // Technology / Tools
  Cpu,
  HardDrive,
  Globe,
  Wifi,
  Terminal,
  Code,
  Database,
  Server,
  Monitor,
  Smartphone,
  Zap,
  Power,
  Battery,
  Key,
  Lock,

  // Objects / Symbols
  Rocket,
  Package,
  FileText,
  BookOpen,
  PenTool,
  Palette,
  Camera,
  Music,
  Bell,
  Clock,
  Compass,
  Map,
  Lightbulb,
  Puzzle,
  Gem,
  Hexagon,
  Circle,
  Triangle,
  Diamond,
  Award,
  Target,
  Anchor,
  Paperclip,
  Flag,
  Gift,
  Hammer,
  Wrench,
  Scissors,
  RefreshCw,

  // Communication
  MessageSquare,
  MessageCircle,
  Mail,
  Phone,
  Mic,
  Volume,
  Megaphone,
  Send,
  Hash,
  AtSign,

  // Arrows / Navigation
  ArrowUp,
  ArrowDown,
  ArrowLeft,
  ArrowRight,
  ChevronUp,
  ChevronDown,
  ExternalLink,
  Navigation,

  // Status / Emotion
  CheckCircle,
  XCircle,
  AlertCircle,
  HelpCircle,
  ThumbsUp,
  ThumbsDown,
  Laugh,
  Frown,
  Eye,
  EyeOff,
} from 'lucide-react';
import type { LucideIcon } from 'lucide-react';

// ---------------------------------------------------------------------------
// Category & Icon Types
// ---------------------------------------------------------------------------

/** Icon category for grouping in the picker */
export interface IconCategory {
  id: string;
  label: string;
  icons: string[];
}

/** Full icon entry */
export interface IconEntry {
  name: string;
  component: LucideIcon;
  category: string;
  keywords: string[];
}

// ---------------------------------------------------------------------------
// Icon Definitions
// ---------------------------------------------------------------------------

const ICON_MAP: Record<string, { component: LucideIcon; keywords: string[]; category: string }> = {
  // --- Characters / People ---
  Bot: { component: Bot, keywords: ['robot', 'agent', 'ai', 'assistant'], category: 'characters' },
  User: { component: User, keywords: ['person', 'human', 'you', 'self'], category: 'characters' },
  Users: { component: Users, keywords: ['people', 'group', 'team', 'community'], category: 'characters' },
  UserCircle: { component: UserCircle, keywords: ['avatar', 'profile', 'account'], category: 'characters' },
  Ghost: { component: Ghost, keywords: ['phantom', 'invisible', 'stealth', ' spooky'], category: 'characters' },
  Skull: { component: Skull, keywords: ['danger', 'death', 'pirate', 'edgy'], category: 'characters' },
  Crown: { component: Crown, keywords: ['king', 'queen', 'royal', 'premium'], category: 'characters' },
  Sparkles: { component: Sparkles, keywords: ['magic', 'shine', 'star', 'special'], category: 'characters' },
  Star: { component: Star, keywords: ['favorite', 'rating', 'gold', 'bright'], category: 'characters' },
  Heart: { component: Heart, keywords: ['love', 'like', 'favorite', 'emotion'], category: 'characters' },
  Shield: { component: Shield, keywords: ['protection', 'security', 'guard', 'defense'], category: 'characters' },
  Swords: { component: Swords, keywords: ['battle', 'fight', 'combat', 'warrior'], category: 'characters' },

  // --- Animals / Nature ---
  Bird: { component: Bird, keywords: ['fly', 'tweet', 'feather', 'freedom'], category: 'nature' },
  Bug: { component: Bug, keywords: ['insect', 'debug', 'crawler', 'ant'], category: 'nature' },
  Cat: { component: Cat, keywords: ['kitten', 'feline', 'pet', 'meow'], category: 'nature' },
  Dog: { component: Dog, keywords: ['puppy', 'canine', 'pet', 'bark'], category: 'nature' },
  Fish: { component: Fish, keywords: ['swim', 'aquatic', 'ocean', 'sea'], category: 'nature' },
  Leaf: { component: Leaf, keywords: ['plant', 'green', 'nature', 'eco'], category: 'nature' },
  TreePine: { component: TreePine, keywords: ['forest', 'tree', 'wood', 'evergreen'], category: 'nature' },
  Flame: { component: Flame, keywords: ['fire', 'hot', 'burn', 'energy'], category: 'nature' },
  Snowflake: { component: Snowflake, keywords: ['cold', 'winter', 'ice', 'frozen'], category: 'nature' },
  Sun: { component: Sun, keywords: ['bright', 'day', 'warm', 'light'], category: 'nature' },
  Moon: { component: Moon, keywords: ['night', 'dark', 'crescent', 'lunar'], category: 'nature' },
  Cloud: { component: Cloud, keywords: ['sky', 'weather', 'rain', 'overcast'], category: 'nature' },
  Rainbow: { component: Rainbow, keywords: ['color', 'arc', 'multicolor', 'hope'], category: 'nature' },
  Mountain: { component: Mountain, keywords: ['peak', 'climb', 'hill', 'landscape'], category: 'nature' },
  Waves: { component: Waves, keywords: ['ocean', 'water', 'surf', 'tide'], category: 'nature' },

  // --- Technology / Tools ---
  Cpu: { component: Cpu, keywords: ['processor', 'chip', 'compute', 'hardware'], category: 'tech' },
  HardDrive: { component: HardDrive, keywords: ['storage', 'disk', 'data', 'memory'], category: 'tech' },
  Globe: { component: Globe, keywords: ['world', 'internet', 'web', 'earth'], category: 'tech' },
  Wifi: { component: Wifi, keywords: ['network', 'wireless', 'connection', 'signal'], category: 'tech' },
  Terminal: { component: Terminal, keywords: ['console', 'command', 'cli', 'shell'], category: 'tech' },
  Code: { component: Code, keywords: ['programming', 'developer', 'script', 'syntax'], category: 'tech' },
  Database: { component: Database, keywords: ['storage', 'records', 'query', 'sql'], category: 'tech' },
  Server: { component: Server, keywords: ['host', 'backend', 'infrastructure', 'rack'], category: 'tech' },
  Monitor: { component: Monitor, keywords: ['screen', 'display', 'desktop', 'computer'], category: 'tech' },
  Smartphone: { component: Smartphone, keywords: ['mobile', 'phone', 'device', 'cell'], category: 'tech' },
  Zap: { component: Zap, keywords: ['lightning', 'power', 'energy', 'fast'], category: 'tech' },
  Power: { component: Power, keywords: ['on', 'off', 'switch', 'energy'], category: 'tech' },
  Battery: { component: Battery, keywords: ['charge', 'power', 'energy', 'level'], category: 'tech' },
  Key: { component: Key, keywords: ['password', 'access', 'security', 'token'], category: 'tech' },
  Lock: { component: Lock, keywords: ['secure', 'private', 'encrypt', 'closed'], category: 'tech' },

  // --- Objects / Symbols ---
  Rocket: { component: Rocket, keywords: ['launch', 'space', 'fast', 'startup'], category: 'objects' },
  Package: { component: Package, keywords: ['box', 'delivery', 'container', 'module'], category: 'objects' },
  FileText: { component: FileText, keywords: ['document', 'file', 'text', 'page'], category: 'objects' },
  BookOpen: { component: BookOpen, keywords: ['read', 'knowledge', 'manual', 'guide'], category: 'objects' },
  PenTool: { component: PenTool, keywords: ['write', 'design', 'draw', 'create'], category: 'objects' },
  Palette: { component: Palette, keywords: ['color', 'art', 'paint', 'design'], category: 'objects' },
  Camera: { component: Camera, keywords: ['photo', 'capture', 'image', 'picture'], category: 'objects' },
  Music: { component: Music, keywords: ['sound', 'audio', 'song', 'note'], category: 'objects' },
  Bell: { component: Bell, keywords: ['notification', 'alert', 'ring', 'chime'], category: 'objects' },
  Clock: { component: Clock, keywords: ['time', 'hour', 'minute', 'watch'], category: 'objects' },
  Compass: { component: Compass, keywords: ['direction', 'navigate', 'explore', 'north'], category: 'objects' },
  Map: { component: Map, keywords: ['location', 'navigate', 'area', 'territory'], category: 'objects' },
  Lightbulb: { component: Lightbulb, keywords: ['idea', 'bright', 'innovation', 'think'], category: 'objects' },
  Puzzle: { component: Puzzle, keywords: ['piece', 'solve', 'game', 'connect'], category: 'objects' },
  Gem: { component: Gem, keywords: ['diamond', 'jewel', 'precious', 'rare'], category: 'objects' },
  Hexagon: { component: Hexagon, keywords: ['shape', 'six', 'geometry', 'hex'], category: 'objects' },
  Circle: { component: Circle, keywords: ['round', 'shape', 'dot', 'cycle'], category: 'objects' },
  Triangle: { component: Triangle, keywords: ['shape', 'three', 'point', 'arrow'], category: 'objects' },
  Diamond: { component: Diamond, keywords: ['shape', 'gem', 'precious', 'rhombus'], category: 'objects' },
  Award: { component: Award, keywords: ['medal', 'prize', 'achievement', 'trophy'], category: 'objects' },
  Target: { component: Target, keywords: ['goal', 'aim', 'bullseye', 'focus'], category: 'objects' },
  Anchor: { component: Anchor, keywords: ['ship', 'sea', 'stable', 'hold'], category: 'objects' },
  Paperclip: { component: Paperclip, keywords: ['attach', 'clip', 'link', 'connect'], category: 'objects' },
  Flag: { component: Flag, keywords: ['marker', 'country', 'signal', 'banner'], category: 'objects' },
  Gift: { component: Gift, keywords: ['present', 'surprise', 'box', 'celebrate'], category: 'objects' },
  Hammer: { component: Hammer, keywords: ['tool', 'build', 'fix', 'construct'], category: 'objects' },
  Wrench: { component: Wrench, keywords: ['tool', 'fix', 'settings', 'repair'], category: 'objects' },
  Scissors: { component: Scissors, keywords: ['cut', 'clip', 'trim', 'edit'], category: 'objects' },
  RefreshCw: { component: RefreshCw, keywords: ['reload', 'sync', 'update', 'repeat'], category: 'objects' },

  // --- Communication ---
  MessageSquare: { component: MessageSquare, keywords: ['chat', 'message', 'comment', 'talk'], category: 'communication' },
  MessageCircle: { component: MessageCircle, keywords: ['chat', 'bubble', 'conversation', 'dialog'], category: 'communication' },
  Mail: { component: Mail, keywords: ['email', 'letter', 'inbox', 'send'], category: 'communication' },
  Phone: { component: Phone, keywords: ['call', 'contact', 'dial', 'ring'], category: 'communication' },
  Mic: { component: Mic, keywords: ['microphone', 'record', 'speak', 'voice'], category: 'communication' },
  Volume: { component: Volume, keywords: ['sound', 'speaker', 'loud', 'audio'], category: 'communication' },
  Megaphone: { component: Megaphone, keywords: ['announce', 'broadcast', 'loud', 'promo'], category: 'communication' },
  Send: { component: Send, keywords: ['submit', 'transmit', 'deliver', 'arrow'], category: 'communication' },
  Hash: { component: Hash, keywords: ['number', 'tag', 'channel', 'pound'], category: 'communication' },
  AtSign: { component: AtSign, keywords: ['mention', 'email', 'at', 'tag'], category: 'communication' },

  // --- Arrows / Navigation ---
  ArrowUp: { component: ArrowUp, keywords: ['up', 'increase', 'rise', 'north'], category: 'navigation' },
  ArrowDown: { component: ArrowDown, keywords: ['down', 'decrease', 'fall', 'south'], category: 'navigation' },
  ArrowLeft: { component: ArrowLeft, keywords: ['left', 'back', 'previous', 'west'], category: 'navigation' },
  ArrowRight: { component: ArrowRight, keywords: ['right', 'forward', 'next', 'east'], category: 'navigation' },
  ChevronUp: { component: ChevronUp, keywords: ['up', 'collapse', 'expand', 'arrow'], category: 'navigation' },
  ChevronDown: { component: ChevronDown, keywords: ['down', 'expand', 'collapse', 'arrow'], category: 'navigation' },
  ExternalLink: { component: ExternalLink, keywords: ['link', 'open', 'url', 'external'], category: 'navigation' },
  Navigation: { component: Navigation, keywords: ['compass', 'location', 'direction', 'map'], category: 'navigation' },

  // --- Status / Emotion ---
  CheckCircle: { component: CheckCircle, keywords: ['success', 'done', 'complete', 'ok'], category: 'status' },
  XCircle: { component: XCircle, keywords: ['error', 'fail', 'wrong', 'close'], category: 'status' },
  AlertCircle: { component: AlertCircle, keywords: ['warning', 'alert', 'caution', 'notice'], category: 'status' },
  HelpCircle: { component: HelpCircle, keywords: ['question', 'support', 'info', 'assist'], category: 'status' },
  ThumbsUp: { component: ThumbsUp, keywords: ['like', 'approve', 'good', 'yes'], category: 'status' },
  ThumbsDown: { component: ThumbsDown, keywords: ['dislike', 'reject', 'bad', 'no'], category: 'status' },
  Laugh: { component: Laugh, keywords: ['happy', 'joy', 'fun', 'smile'], category: 'status' },
  Frown: { component: Frown, keywords: ['sad', 'unhappy', 'upset', 'disappointed'], category: 'status' },
  Eye: { component: Eye, keywords: ['view', 'watch', 'see', 'visible'], category: 'status' },
  EyeOff: { component: EyeOff, keywords: ['hidden', 'invisible', 'private', 'blind'], category: 'status' },
};

// ---------------------------------------------------------------------------
// Categories Definition
// ---------------------------------------------------------------------------

/** Ordered list of icon categories for the picker */
export const ICON_CATEGORIES: IconCategory[] = [
  { id: 'characters', label: 'Characters', icons: Object.keys(ICON_MAP).filter(k => ICON_MAP[k].category === 'characters') },
  { id: 'nature', label: 'Nature', icons: Object.keys(ICON_MAP).filter(k => ICON_MAP[k].category === 'nature') },
  { id: 'tech', label: 'Tech', icons: Object.keys(ICON_MAP).filter(k => ICON_MAP[k].category === 'tech') },
  { id: 'objects', label: 'Objects', icons: Object.keys(ICON_MAP).filter(k => ICON_MAP[k].category === 'objects') },
  { id: 'communication', label: 'Chat', icons: Object.keys(ICON_MAP).filter(k => ICON_MAP[k].category === 'communication') },
  { id: 'navigation', label: 'Arrows', icons: Object.keys(ICON_MAP).filter(k => ICON_MAP[k].category === 'navigation') },
  { id: 'status', label: 'Status', icons: Object.keys(ICON_MAP).filter(k => ICON_MAP[k].category === 'status') },
];

// ---------------------------------------------------------------------------
// Lookup Helpers
// ---------------------------------------------------------------------------

/** Get the LucideIcon component for a given icon name */
export function getIconComponent(name: string): LucideIcon | null {
  return ICON_MAP[name]?.component ?? null;
}

/** Get the category for a given icon name */
export function getIconCategory(name: string): string {
  return ICON_MAP[name]?.category ?? 'objects';
}

/** Get keywords for a given icon name */
export function getIconKeywords(name: string): string[] {
  return ICON_MAP[name]?.keywords ?? [];
}

/** Search icons by query string (matches name and keywords) */
export function searchIcons(query: string): string[] {
  const q = query.toLowerCase().trim();
  if (!q) return Object.keys(ICON_MAP);
  return Object.entries(ICON_MAP)
    .filter(([name, entry]) => {
      if (name.toLowerCase().includes(q)) return true;
      return entry.keywords.some(kw => kw.includes(q));
    })
    .map(([name]) => name);
}

/** Get all icon names */
export function getAllIconNames(): string[] {
  return Object.keys(ICON_MAP);
}

/** Check if an icon name is valid */
export function isValidIconName(name: string): boolean {
  return name in ICON_MAP;
}

/** Default icon for agents */
export const DEFAULT_AGENT_ICON = 'Bot';

/** Default icon for users */
export const DEFAULT_USER_ICON = 'User';
