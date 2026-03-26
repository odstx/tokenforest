import { writable } from 'svelte/store';
import { browser } from '$app/environment';

interface User {
  id: number;
  username: string;
}

interface AuthState {
  token: string | null;
  user: User | null;
}

function createAuthStore() {
  const getInitialState = (): AuthState => {
    if (!browser) return { token: null, user: null };
    
    const token = localStorage.getItem('token');
    const userStr = localStorage.getItem('user');
    
    return {
      token,
      user: userStr ? JSON.parse(userStr) : null
    };
  };

  const { subscribe, set, update } = writable<AuthState>(getInitialState());

  return {
    subscribe,
    login: (token: string, user: User) => {
      if (browser) {
        localStorage.setItem('token', token);
        localStorage.setItem('user', JSON.stringify(user));
      }
      set({ token, user });
    },
    logout: () => {
      if (browser) {
        localStorage.removeItem('token');
        localStorage.removeItem('user');
      }
      set({ token: null, user: null });
    },
    isAuthenticated: () => {
      let state: AuthState;
      const unsubscribe = subscribe(s => state = s);
      unsubscribe();
      return !!state!.token;
    }
  };
}

export const auth = createAuthStore();
