/* eslint-env node */
module.exports = {
    parser: '@typescript-eslint/parser',
    plugins: ['@typescript-eslint', 'import'],
    extends: [
      'eslint:recommended',
      'plugin:@typescript-eslint/recommended',
      'plugin:import/errors',
      'plugin:import/warnings',
      'plugin:import/typescript',
    ],
    rules: {
      'import/order': [
        'error',
        {
          'groups': ['builtin', 'external', 'internal', 'parent', 'sibling', 'index'],
          'newlines-between': 'always',
          'alphabetize': { 'order': 'asc', 'caseInsensitive': true }
        }
      ]
    },
    settings: {
      'import/resolver': {
        typescript: {},
        alias: {
          map: [['@', './src']],
          extensions: ['.ts', '.js', '.jsx', '.json', '.tsx']
        }
      }
    },
    overrides: [
      {
        files: ['next.config.js'],
        env: {
          node: true,
        },
      },
    ],
  };