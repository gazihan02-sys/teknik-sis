/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.rs",
    "./index.html",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        // MD3 Primary — CSS variable driven for light/dark
        primary: {
          DEFAULT: 'var(--md-primary)',
          container: 'var(--md-primary-container)',
          on: 'var(--md-on-primary)',
          'on-container': 'var(--md-on-primary-container)',
        },
        // MD3 Secondary
        secondary: {
          DEFAULT: 'var(--md-secondary)',
          container: 'var(--md-secondary-container)',
          on: 'var(--md-on-secondary)',
          'on-container': 'var(--md-on-secondary-container)',
        },
        // MD3 Tertiary
        tertiary: {
          DEFAULT: 'var(--md-tertiary)',
          container: 'var(--md-tertiary-container)',
          on: 'var(--md-on-tertiary)',
          'on-container': 'var(--md-on-tertiary-container)',
        },
        // MD3 Error
        error: {
          DEFAULT: 'var(--md-error)',
          container: 'var(--md-error-container)',
          on: 'var(--md-on-error)',
          'on-container': 'var(--md-on-error-container)',
        },
        // MD3 Surface
        surface: {
          DEFAULT: 'var(--md-surface)',
          dim: 'var(--md-surface-dim)',
          bright: 'var(--md-surface-bright)',
          'container-lowest': 'var(--md-surface-container-lowest)',
          'container-low': 'var(--md-surface-container-low)',
          container: 'var(--md-surface-container)',
          'container-high': 'var(--md-surface-container-high)',
          'container-highest': 'var(--md-surface-container-highest)',
          on: 'var(--md-on-surface)',
          'on-variant': 'var(--md-on-surface-variant)',
          inverse: 'var(--md-inverse-surface)',
          'inverse-on': 'var(--md-inverse-on-surface)',
        },
        // MD3 Outline
        outline: {
          DEFAULT: 'var(--md-outline)',
          variant: 'var(--md-outline-variant)',
        },
      },
      borderRadius: {
        'xs': '4px',
        'sm': '8px',
        'md': '12px',
        'lg': '16px',
        'xl': '28px',
        'full': '9999px',
      },
      fontFamily: {
        sans: ['"Google Sans"', '"Roboto"', 'system-ui', 'sans-serif'],
      },
      fontSize: {
        'display-lg': ['57px', { lineHeight: '64px', letterSpacing: '-0.25px', fontWeight: '400' }],
        'display-md': ['45px', { lineHeight: '52px', letterSpacing: '0px', fontWeight: '400' }],
        'display-sm': ['36px', { lineHeight: '44px', letterSpacing: '0px', fontWeight: '400' }],
        'headline-lg': ['32px', { lineHeight: '40px', letterSpacing: '0px', fontWeight: '400' }],
        'headline-md': ['28px', { lineHeight: '36px', letterSpacing: '0px', fontWeight: '400' }],
        'headline-sm': ['24px', { lineHeight: '32px', letterSpacing: '0px', fontWeight: '400' }],
        'title-lg': ['22px', { lineHeight: '28px', letterSpacing: '0px', fontWeight: '400' }],
        'title-md': ['16px', { lineHeight: '24px', letterSpacing: '0.15px', fontWeight: '500' }],
        'title-sm': ['14px', { lineHeight: '20px', letterSpacing: '0.1px', fontWeight: '500' }],
        'body-lg': ['16px', { lineHeight: '24px', letterSpacing: '0.5px', fontWeight: '400' }],
        'body-md': ['14px', { lineHeight: '20px', letterSpacing: '0.25px', fontWeight: '400' }],
        'body-sm': ['12px', { lineHeight: '16px', letterSpacing: '0.4px', fontWeight: '400' }],
        'label-lg': ['14px', { lineHeight: '20px', letterSpacing: '0.1px', fontWeight: '500' }],
        'label-md': ['12px', { lineHeight: '16px', letterSpacing: '0.5px', fontWeight: '500' }],
        'label-sm': ['11px', { lineHeight: '16px', letterSpacing: '0.5px', fontWeight: '500' }],
      },
      boxShadow: {
        'elevation-1': '0px 1px 2px var(--md-shadow-color, rgba(0,0,0,0.3)), 0px 1px 3px 1px var(--md-shadow-color, rgba(0,0,0,0.15))',
        'elevation-2': '0px 1px 2px var(--md-shadow-color, rgba(0,0,0,0.3)), 0px 2px 6px 2px var(--md-shadow-color, rgba(0,0,0,0.15))',
        'elevation-3': '0px 4px 8px 3px var(--md-shadow-color, rgba(0,0,0,0.15)), 0px 1px 3px var(--md-shadow-color, rgba(0,0,0,0.3))',
        'elevation-4': '0px 6px 10px 4px var(--md-shadow-color, rgba(0,0,0,0.15)), 0px 2px 3px var(--md-shadow-color, rgba(0,0,0,0.3))',
        'elevation-5': '0px 8px 12px 6px var(--md-shadow-color, rgba(0,0,0,0.15)), 0px 4px 4px var(--md-shadow-color, rgba(0,0,0,0.3))',
      },
    },
  },
  plugins: [],
}
