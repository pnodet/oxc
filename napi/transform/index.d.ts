/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export interface IsolatedDeclarationsResult {
  sourceText: string
  errors: Array<string>
}
/** TypeScript Isolated Declarations for Standalone DTS Emit */
function isolatedDeclaration(filename: string, sourceText: string): IsolatedDeclarationsResult
export interface TypeScriptBindingOptions {
  jsxPragma?: string
  jsxPragmaFrag?: string
  onlyRemoveTypeImports?: boolean
  allowNamespaces?: boolean
  allowDeclareFields?: boolean
}
export interface ReactBindingOptions {
  runtime?: 'classic' | 'automatic'
  development?: boolean
  throwIfNamespace?: boolean
  pure?: boolean
  importSource?: string
  pragma?: string
  pragmaFrag?: string
  useBuiltIns?: boolean
  useSpread?: boolean
}
export interface ArrowFunctionsBindingOptions {
  spec?: boolean
}
export interface Es2015BindingOptions {
  arrowFunction?: ArrowFunctionsBindingOptions
}
export interface TransformOptions {
  sourceType?: 'script' | 'module' | 'unambiguous' | undefined
  /** Force jsx parsing, */
  jsx?: boolean
  typescript?: TypeScriptBindingOptions
  react?: ReactBindingOptions
  es2015?: Es2015BindingOptions
  /**
   * Enable Sourcemap
   *
   * * `true` to generate a sourcemap for the code and include it in the result object.
   *
   * Default: false
   */
  sourcemap?: boolean
}
export interface Sourcemap {
  file?: string
  mappings?: string
  sourceRoot?: string
  sources?: Array<string | undefined | null>
  sourcesContent?: Array<string | undefined | null>
  names?: Array<string>
}
export interface TransformResult {
  sourceText: string
  map?: Sourcemap
  errors: Array<string>
}
function transform(filename: string, sourceText: string, options?: TransformOptions | undefined | null): TransformResult
