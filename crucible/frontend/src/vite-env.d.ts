/// <reference types="vite/client" />
import type React from 'react';

declare global {
  namespace JSX {
    interface IntrinsicElements {
      'chaos-vault': React.DetailedHTMLProps<React.HTMLAttributes<HTMLElement>, HTMLElement>;
    }
  }
}
