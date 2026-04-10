import { useState, useCallback, useEffect } from 'react';

export interface UserProfile {
  name: string;
  email: string;
}

const STORAGE_KEY = 'slockai_user_profile';

const DEFAULT_PROFILE: UserProfile = {
  name: 'User',
  email: '',
};

function loadProfile(): UserProfile {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      return { ...DEFAULT_PROFILE, ...JSON.parse(raw) };
    }
  } catch {
    // ignore parse errors
  }
  return { ...DEFAULT_PROFILE };
}

function saveProfile(profile: UserProfile): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(profile));
}

export function useUserProfile() {
  const [profile, setProfile] = useState<UserProfile>(loadProfile);

  useEffect(() => {
    // Sync across tabs
    const handler = (e: StorageEvent) => {
      if (e.key === STORAGE_KEY) {
        setProfile(loadProfile());
      }
    };
    window.addEventListener('storage', handler);
    return () => window.removeEventListener('storage', handler);
  }, []);

  const updateProfile = useCallback((updates: Partial<UserProfile>) => {
    setProfile((prev) => {
      const next = { ...prev, ...updates };
      saveProfile(next);
      return next;
    });
  }, []);

  return { profile, updateProfile };
}
