import typescript from '@rollup/plugin-typescript';
import babel from '@rollup/plugin-babel';

export default {
  input: 'include/js/index.ts',

  output: {
    file: 'include/jsdist/index.js',
    format: 'iife',
    sourcemap: false
  },

  plugins: [
    typescript({
      target: 'ES2019',
      module: 'ESNext'
    }),

    babel({
      babelHelpers: 'bundled',
      extensions: ['.js', '.ts'],
      presets: [
        ['@babel/preset-env', {
          targets: {
            esmodules: false
          }
        }]
      ]
    })
  ]
};
