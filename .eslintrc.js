module.exports = {
  env: {
    browser: true,
    es2021: true,
    node: true
  },
  extends: [
    'semistandard',
    'plugin:react/recommended'
  ],
  overrides: [
    {
      files: ['*.ts', '*.tsx'],
      extends: ['standard-with-typescript'],
      parserOptions: {
        project: ['./tsconfig.json']
      },
      rules: {
        '@typescript-eslint/semi': [2, 'always']
      }
    }
  ],
  parserOptions: {
    ecmaVersion: 'latest',
    sourceType: 'module'
  },
  plugins: [
    'react'
  ],
  rules: {}
};
