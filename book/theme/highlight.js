/*!
  Highlight.js v11.10.0 (git: 366a8bd012)
  (c) 2006-2024 Josh Goebel <hello@joshgoebel.com> and other contributors
  License: BSD-3-Clause
 */
var hljs=function(){"use strict";function e(t){
return t instanceof Map?t.clear=t.delete=t.set=()=>{
throw Error("map is read-only")}:t instanceof Set&&(t.add=t.clear=t.delete=()=>{
throw Error("set is read-only")
}),Object.freeze(t),Object.getOwnPropertyNames(t).forEach((n=>{
const i=t[n],s=typeof i;"object"!==s&&"function"!==s||Object.isFrozen(i)||e(i)
})),t}class t{constructor(e){
void 0===e.data&&(e.data={}),this.data=e.data,this.isMatchIgnored=!1}
ignoreMatch(){this.isMatchIgnored=!0}}function n(e){
return e.replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;").replace(/"/g,"&quot;").replace(/'/g,"&#x27;")
}function i(e,...t){const n=Object.create(null);for(const t in e)n[t]=e[t]
;return t.forEach((e=>{for(const t in e)n[t]=e[t]})),n}const s=e=>!!e.scope
;class o{constructor(e,t){
this.buffer="",this.classPrefix=t.classPrefix,e.walk(this)}addText(e){
this.buffer+=n(e)}openNode(e){if(!s(e))return;const t=((e,{prefix:t})=>{
if(e.startsWith("language:"))return e.replace("language:","language-")
;if(e.includes(".")){const n=e.split(".")
;return[`${t}${n.shift()}`,...n.map(((e,t)=>`${e}${"_".repeat(t+1)}`))].join(" ")
}return`${t}${e}`})(e.scope,{prefix:this.classPrefix});this.span(t)}
closeNode(e){s(e)&&(this.buffer+="</span>")}value(){return this.buffer}span(e){
this.buffer+=`<span class="${e}">`}}const r=(e={})=>{const t={children:[]}
;return Object.assign(t,e),t};class a{constructor(){
this.rootNode=r(),this.stack=[this.rootNode]}get top(){
return this.stack[this.stack.length-1]}get root(){return this.rootNode}add(e){
this.top.children.push(e)}openNode(e){const t=r({scope:e})
;this.add(t),this.stack.push(t)}closeNode(){
if(this.stack.length>1)return this.stack.pop()}closeAllNodes(){
for(;this.closeNode(););}toJSON(){return JSON.stringify(this.rootNode,null,4)}
walk(e){return this.constructor._walk(e,this.rootNode)}static _walk(e,t){
return"string"==typeof t?e.addText(t):t.children&&(e.openNode(t),
t.children.forEach((t=>this._walk(e,t))),e.closeNode(t)),e}static _collapse(e){
"string"!=typeof e&&e.children&&(e.children.every((e=>"string"==typeof e))?e.children=[e.children.join("")]:e.children.forEach((e=>{
a._collapse(e)})))}}class c extends a{constructor(e){super(),this.options=e}
addText(e){""!==e&&this.add(e)}startScope(e){this.openNode(e)}endScope(){
this.closeNode()}__addSublanguage(e,t){const n=e.root
;t&&(n.scope="language:"+t),this.add(n)}toHTML(){
return new o(this,this.options).value()}finalize(){
return this.closeAllNodes(),!0}}function l(e){
return e?"string"==typeof e?e:e.source:null}function g(e){return h("(?=",e,")")}
function u(e){return h("(?:",e,")*")}function d(e){return h("(?:",e,")?")}
function h(...e){return e.map((e=>l(e))).join("")}function f(...e){const t=(e=>{
const t=e[e.length-1]
;return"object"==typeof t&&t.constructor===Object?(e.splice(e.length-1,1),t):{}
})(e);return"("+(t.capture?"":"?:")+e.map((e=>l(e))).join("|")+")"}
function p(e){return RegExp(e.toString()+"|").exec("").length-1}
const b=/\[(?:[^\\\]]|\\.)*\]|\(\??|\\([1-9][0-9]*)|\\./
;function m(e,{joinWith:t}){let n=0;return e.map((e=>{n+=1;const t=n
;let i=l(e),s="";for(;i.length>0;){const e=b.exec(i);if(!e){s+=i;break}
s+=i.substring(0,e.index),
i=i.substring(e.index+e[0].length),"\\"===e[0][0]&&e[1]?s+="\\"+(Number(e[1])+t):(s+=e[0],
"("===e[0]&&n++)}return s})).map((e=>`(${e})`)).join(t)}
const E="[a-zA-Z]\\w*",x="[a-zA-Z_]\\w*",w="\\b\\d+(\\.\\d+)?",y="(-?)(\\b0[xX][a-fA-F0-9]+|(\\b\\d+(\\.\\d*)?|\\.\\d+)([eE][-+]?\\d+)?)",_="\\b(0b[01]+)",O={
begin:"\\\\[\\s\\S]",relevance:0},v={scope:"string",begin:"'",end:"'",
illegal:"\\n",contains:[O]},k={scope:"string",begin:'"',end:'"',illegal:"\\n",
contains:[O]},N=(e,t,n={})=>{const s=i({scope:"comment",begin:e,end:t,
contains:[]},n);s.contains.push({scope:"doctag",
begin:"[ ]*(?=(TODO|FIXME|NOTE|BUG|OPTIMIZE|HACK|XXX):)",
end:/(TODO|FIXME|NOTE|BUG|OPTIMIZE|HACK|XXX):/,excludeBegin:!0,relevance:0})
;const o=f("I","a","is","so","us","to","at","if","in","it","on",/[A-Za-z]+['](d|ve|re|ll|t|s|n)/,/[A-Za-z]+[-][a-z]+/,/[A-Za-z][a-z]{2,}/)
;return s.contains.push({begin:h(/[ ]+/,"(",o,/[.]?[:]?([.][ ]|[ ])/,"){3}")}),s
},S=N("//","$"),M=N("/\\*","\\*/"),R=N("#","$");var j=Object.freeze({
__proto__:null,APOS_STRING_MODE:v,BACKSLASH_ESCAPE:O,BINARY_NUMBER_MODE:{
scope:"number",begin:_,relevance:0},BINARY_NUMBER_RE:_,COMMENT:N,
C_BLOCK_COMMENT_MODE:M,C_LINE_COMMENT_MODE:S,C_NUMBER_MODE:{scope:"number",
begin:y,relevance:0},C_NUMBER_RE:y,END_SAME_AS_BEGIN:e=>Object.assign(e,{
"on:begin":(e,t)=>{t.data._beginMatch=e[1]},"on:end":(e,t)=>{
t.data._beginMatch!==e[1]&&t.ignoreMatch()}}),HASH_COMMENT_MODE:R,IDENT_RE:E,
MATCH_NOTHING_RE:/\b\B/,METHOD_GUARD:{begin:"\\.\\s*"+x,relevance:0},
NUMBER_MODE:{scope:"number",begin:w,relevance:0},NUMBER_RE:w,
PHRASAL_WORDS_MODE:{
begin:/\b(a|an|the|are|I'm|isn't|don't|doesn't|won't|but|just|should|pretty|simply|enough|gonna|going|wtf|so|such|will|you|your|they|like|more)\b/
},QUOTE_STRING_MODE:k,REGEXP_MODE:{scope:"regexp",begin:/\/(?=[^/\n]*\/)/,
end:/\/[gimuy]*/,contains:[O,{begin:/\[/,end:/\]/,relevance:0,contains:[O]}]},
RE_STARTERS_RE:"!|!=|!==|%|%=|&|&&|&=|\\*|\\*=|\\+|\\+=|,|-|-=|/=|/|:|;|<<|<<=|<=|<|===|==|=|>>>=|>>=|>=|>>>|>>|>|\\?|\\[|\\{|\\(|\\^|\\^=|\\||\\|=|\\|\\||~",
SHEBANG:(e={})=>{const t=/^#![ ]*\//
;return e.binary&&(e.begin=h(t,/.*\b/,e.binary,/\b.*/)),i({scope:"meta",begin:t,
end:/$/,relevance:0,"on:begin":(e,t)=>{0!==e.index&&t.ignoreMatch()}},e)},
TITLE_MODE:{scope:"title",begin:E,relevance:0},UNDERSCORE_IDENT_RE:x,
UNDERSCORE_TITLE_MODE:{scope:"title",begin:x,relevance:0}});function A(e,t){
"."===e.input[e.index-1]&&t.ignoreMatch()}function I(e,t){
void 0!==e.className&&(e.scope=e.className,delete e.className)}function T(e,t){
t&&e.beginKeywords&&(e.begin="\\b("+e.beginKeywords.split(" ").join("|")+")(?!\\.)(?=\\b|\\s)",
e.__beforeBegin=A,e.keywords=e.keywords||e.beginKeywords,delete e.beginKeywords,
void 0===e.relevance&&(e.relevance=0))}function L(e,t){
Array.isArray(e.illegal)&&(e.illegal=f(...e.illegal))}function B(e,t){
if(e.match){
if(e.begin||e.end)throw Error("begin & end are not supported with match")
;e.begin=e.match,delete e.match}}function P(e,t){
void 0===e.relevance&&(e.relevance=1)}const D=(e,t)=>{if(!e.beforeMatch)return
;if(e.starts)throw Error("beforeMatch cannot be used with starts")
;const n=Object.assign({},e);Object.keys(e).forEach((t=>{delete e[t]
})),e.keywords=n.keywords,e.begin=h(n.beforeMatch,g(n.begin)),e.starts={
relevance:0,contains:[Object.assign(n,{endsParent:!0})]
},e.relevance=0,delete n.beforeMatch
},H=["of","and","for","in","not","or","if","then","parent","list","value"],C="keyword"
;function $(e,t,n=C){const i=Object.create(null)
;return"string"==typeof e?s(n,e.split(" ")):Array.isArray(e)?s(n,e):Object.keys(e).forEach((n=>{
Object.assign(i,$(e[n],t,n))})),i;function s(e,n){
t&&(n=n.map((e=>e.toLowerCase()))),n.forEach((t=>{const n=t.split("|")
;i[n[0]]=[e,U(n[0],n[1])]}))}}function U(e,t){
return t?Number(t):(e=>H.includes(e.toLowerCase()))(e)?0:1}const z={},W=e=>{
console.error(e)},X=(e,...t)=>{console.log("WARN: "+e,...t)},G=(e,t)=>{
z[`${e}/${t}`]||(console.log(`Deprecated as of ${e}. ${t}`),z[`${e}/${t}`]=!0)
},K=Error();function F(e,t,{key:n}){let i=0;const s=e[n],o={},r={}
;for(let e=1;e<=t.length;e++)r[e+i]=s[e],o[e+i]=!0,i+=p(t[e-1])
;e[n]=r,e[n]._emit=o,e[n]._multi=!0}function Z(e){(e=>{
e.scope&&"object"==typeof e.scope&&null!==e.scope&&(e.beginScope=e.scope,
delete e.scope)})(e),"string"==typeof e.beginScope&&(e.beginScope={
_wrap:e.beginScope}),"string"==typeof e.endScope&&(e.endScope={_wrap:e.endScope
}),(e=>{if(Array.isArray(e.begin)){
if(e.skip||e.excludeBegin||e.returnBegin)throw W("skip, excludeBegin, returnBegin not compatible with beginScope: {}"),
K
;if("object"!=typeof e.beginScope||null===e.beginScope)throw W("beginScope must be object"),
K;F(e,e.begin,{key:"beginScope"}),e.begin=m(e.begin,{joinWith:""})}})(e),(e=>{
if(Array.isArray(e.end)){
if(e.skip||e.excludeEnd||e.returnEnd)throw W("skip, excludeEnd, returnEnd not compatible with endScope: {}"),
K
;if("object"!=typeof e.endScope||null===e.endScope)throw W("endScope must be object"),
K;F(e,e.end,{key:"endScope"}),e.end=m(e.end,{joinWith:""})}})(e)}function V(e){
function t(t,n){
return RegExp(l(t),"m"+(e.case_insensitive?"i":"")+(e.unicodeRegex?"u":"")+(n?"g":""))
}class n{constructor(){
this.matchIndexes={},this.regexes=[],this.matchAt=1,this.position=0}
addRule(e,t){
t.position=this.position++,this.matchIndexes[this.matchAt]=t,this.regexes.push([t,e]),
this.matchAt+=p(e)+1}compile(){0===this.regexes.length&&(this.exec=()=>null)
;const e=this.regexes.map((e=>e[1]));this.matcherRe=t(m(e,{joinWith:"|"
}),!0),this.lastIndex=0}exec(e){this.matcherRe.lastIndex=this.lastIndex
;const t=this.matcherRe.exec(e);if(!t)return null
;const n=t.findIndex(((e,t)=>t>0&&void 0!==e)),i=this.matchIndexes[n]
;return t.splice(0,n),Object.assign(t,i)}}class s{constructor(){
this.rules=[],this.multiRegexes=[],
this.count=0,this.lastIndex=0,this.regexIndex=0}getMatcher(e){
if(this.multiRegexes[e])return this.multiRegexes[e];const t=new n
;return this.rules.slice(e).forEach((([e,n])=>t.addRule(e,n))),
t.compile(),this.multiRegexes[e]=t,t}resumingScanAtSamePosition(){
return 0!==this.regexIndex}considerAll(){this.regexIndex=0}addRule(e,t){
this.rules.push([e,t]),"begin"===t.type&&this.count++}exec(e){
const t=this.getMatcher(this.regexIndex);t.lastIndex=this.lastIndex
;let n=t.exec(e)
;if(this.resumingScanAtSamePosition())if(n&&n.index===this.lastIndex);else{
const t=this.getMatcher(0);t.lastIndex=this.lastIndex+1,n=t.exec(e)}
return n&&(this.regexIndex+=n.position+1,
this.regexIndex===this.count&&this.considerAll()),n}}
if(e.compilerExtensions||(e.compilerExtensions=[]),
e.contains&&e.contains.includes("self"))throw Error("ERR: contains `self` is not supported at the top-level of a language.  See documentation.")
;return e.classNameAliases=i(e.classNameAliases||{}),function n(o,r){const a=o
;if(o.isCompiled)return a
;[I,B,Z,D].forEach((e=>e(o,r))),e.compilerExtensions.forEach((e=>e(o,r))),
o.__beforeBegin=null,[T,L,P].forEach((e=>e(o,r))),o.isCompiled=!0;let c=null
;return"object"==typeof o.keywords&&o.keywords.$pattern&&(o.keywords=Object.assign({},o.keywords),
c=o.keywords.$pattern,
delete o.keywords.$pattern),c=c||/\w+/,o.keywords&&(o.keywords=$(o.keywords,e.case_insensitive)),
a.keywordPatternRe=t(c,!0),
r&&(o.begin||(o.begin=/\B|\b/),a.beginRe=t(a.begin),o.end||o.endsWithParent||(o.end=/\B|\b/),
o.end&&(a.endRe=t(a.end)),
a.terminatorEnd=l(a.end)||"",o.endsWithParent&&r.terminatorEnd&&(a.terminatorEnd+=(o.end?"|":"")+r.terminatorEnd)),
o.illegal&&(a.illegalRe=t(o.illegal)),
o.contains||(o.contains=[]),o.contains=[].concat(...o.contains.map((e=>(e=>(e.variants&&!e.cachedVariants&&(e.cachedVariants=e.variants.map((t=>i(e,{
variants:null},t)))),e.cachedVariants?e.cachedVariants:q(e)?i(e,{
starts:e.starts?i(e.starts):null
}):Object.isFrozen(e)?i(e):e))("self"===e?o:e)))),o.contains.forEach((e=>{n(e,a)
})),o.starts&&n(o.starts,r),a.matcher=(e=>{const t=new s
;return e.contains.forEach((e=>t.addRule(e.begin,{rule:e,type:"begin"
}))),e.terminatorEnd&&t.addRule(e.terminatorEnd,{type:"end"
}),e.illegal&&t.addRule(e.illegal,{type:"illegal"}),t})(a),a}(e)}function q(e){
return!!e&&(e.endsWithParent||q(e.starts))}class J extends Error{
constructor(e,t){super(e),this.name="HTMLInjectionError",this.html=t}}
const Y=n,Q=i,ee=Symbol("nomatch"),te=n=>{
const i=Object.create(null),s=Object.create(null),o=[];let r=!0
;const a="Could not find the language '{}', did you forget to load/include a language module?",l={
disableAutodetect:!0,name:"Plain text",contains:[]};let p={
ignoreUnescapedHTML:!1,throwUnescapedHTML:!1,noHighlightRe:/^(no-?highlight)$/i,
languageDetectRe:/\blang(?:uage)?-([\w-]+)\b/i,classPrefix:"hljs-",
cssSelector:"pre code",languages:null,__emitter:c};function b(e){
return p.noHighlightRe.test(e)}function m(e,t,n){let i="",s=""
;"object"==typeof t?(i=e,
n=t.ignoreIllegals,s=t.language):(G("10.7.0","highlight(lang, code, ...args) has been deprecated."),
G("10.7.0","Please use highlight(code, options) instead.\nhttps://github.com/highlightjs/highlight.js/issues/2277"),
s=e,i=t),void 0===n&&(n=!0);const o={code:i,language:s};N("before:highlight",o)
;const r=o.result?o.result:E(o.language,o.code,n)
;return r.code=o.code,N("after:highlight",r),r}function E(e,n,s,o){
const c=Object.create(null);function l(){if(!N.keywords)return void M.addText(R)
;let e=0;N.keywordPatternRe.lastIndex=0;let t=N.keywordPatternRe.exec(R),n=""
;for(;t;){n+=R.substring(e,t.index)
;const s=_.case_insensitive?t[0].toLowerCase():t[0],o=(i=s,N.keywords[i]);if(o){
const[e,i]=o
;if(M.addText(n),n="",c[s]=(c[s]||0)+1,c[s]<=7&&(j+=i),e.startsWith("_"))n+=t[0];else{
const n=_.classNameAliases[e]||e;u(t[0],n)}}else n+=t[0]
;e=N.keywordPatternRe.lastIndex,t=N.keywordPatternRe.exec(R)}var i
;n+=R.substring(e),M.addText(n)}function g(){null!=N.subLanguage?(()=>{
if(""===R)return;let e=null;if("string"==typeof N.subLanguage){
if(!i[N.subLanguage])return void M.addText(R)
;e=E(N.subLanguage,R,!0,S[N.subLanguage]),S[N.subLanguage]=e._top
}else e=x(R,N.subLanguage.length?N.subLanguage:null)
;N.relevance>0&&(j+=e.relevance),M.__addSublanguage(e._emitter,e.language)
})():l(),R=""}function u(e,t){
""!==e&&(M.startScope(t),M.addText(e),M.endScope())}function d(e,t){let n=1
;const i=t.length-1;for(;n<=i;){if(!e._emit[n]){n++;continue}
const i=_.classNameAliases[e[n]]||e[n],s=t[n];i?u(s,i):(R=s,l(),R=""),n++}}
function h(e,t){
return e.scope&&"string"==typeof e.scope&&M.openNode(_.classNameAliases[e.scope]||e.scope),
e.beginScope&&(e.beginScope._wrap?(u(R,_.classNameAliases[e.beginScope._wrap]||e.beginScope._wrap),
R=""):e.beginScope._multi&&(d(e.beginScope,t),R="")),N=Object.create(e,{parent:{
value:N}}),N}function f(e,n,i){let s=((e,t)=>{const n=e&&e.exec(t)
;return n&&0===n.index})(e.endRe,i);if(s){if(e["on:end"]){const i=new t(e)
;e["on:end"](n,i),i.isMatchIgnored&&(s=!1)}if(s){
for(;e.endsParent&&e.parent;)e=e.parent;return e}}
if(e.endsWithParent)return f(e.parent,n,i)}function b(e){
return 0===N.matcher.regexIndex?(R+=e[0],1):(T=!0,0)}function m(e){
const t=e[0],i=n.substring(e.index),s=f(N,e,i);if(!s)return ee;const o=N
;N.endScope&&N.endScope._wrap?(g(),
u(t,N.endScope._wrap)):N.endScope&&N.endScope._multi?(g(),
d(N.endScope,e)):o.skip?R+=t:(o.returnEnd||o.excludeEnd||(R+=t),
g(),o.excludeEnd&&(R=t));do{
N.scope&&M.closeNode(),N.skip||N.subLanguage||(j+=N.relevance),N=N.parent
}while(N!==s.parent);return s.starts&&h(s.starts,e),o.returnEnd?0:t.length}
let w={};function y(i,o){const a=o&&o[0];if(R+=i,null==a)return g(),0
;if("begin"===w.type&&"end"===o.type&&w.index===o.index&&""===a){
if(R+=n.slice(o.index,o.index+1),!r){const t=Error(`0 width match regex (${e})`)
;throw t.languageName=e,t.badRule=w.rule,t}return 1}
if(w=o,"begin"===o.type)return(e=>{
const n=e[0],i=e.rule,s=new t(i),o=[i.__beforeBegin,i["on:begin"]]
;for(const t of o)if(t&&(t(e,s),s.isMatchIgnored))return b(n)
;return i.skip?R+=n:(i.excludeBegin&&(R+=n),
g(),i.returnBegin||i.excludeBegin||(R=n)),h(i,e),i.returnBegin?0:n.length})(o)
;if("illegal"===o.type&&!s){
const e=Error('Illegal lexeme "'+a+'" for mode "'+(N.scope||"<unnamed>")+'"')
;throw e.mode=N,e}if("end"===o.type){const e=m(o);if(e!==ee)return e}
if("illegal"===o.type&&""===a)return 1
;if(I>1e5&&I>3*o.index)throw Error("potential infinite loop, way more iterations than matches")
;return R+=a,a.length}const _=O(e)
;if(!_)throw W(a.replace("{}",e)),Error('Unknown language: "'+e+'"')
;const v=V(_);let k="",N=o||v;const S={},M=new p.__emitter(p);(()=>{const e=[]
;for(let t=N;t!==_;t=t.parent)t.scope&&e.unshift(t.scope)
;e.forEach((e=>M.openNode(e)))})();let R="",j=0,A=0,I=0,T=!1;try{
if(_.__emitTokens)_.__emitTokens(n,M);else{for(N.matcher.considerAll();;){
I++,T?T=!1:N.matcher.considerAll(),N.matcher.lastIndex=A
;const e=N.matcher.exec(n);if(!e)break;const t=y(n.substring(A,e.index),e)
;A=e.index+t}y(n.substring(A))}return M.finalize(),k=M.toHTML(),{language:e,
value:k,relevance:j,illegal:!1,_emitter:M,_top:N}}catch(t){
if(t.message&&t.message.includes("Illegal"))return{language:e,value:Y(n),
illegal:!0,relevance:0,_illegalBy:{message:t.message,index:A,
context:n.slice(A-100,A+100),mode:t.mode,resultSoFar:k},_emitter:M};if(r)return{
language:e,value:Y(n),illegal:!1,relevance:0,errorRaised:t,_emitter:M,_top:N}
;throw t}}function x(e,t){t=t||p.languages||Object.keys(i);const n=(e=>{
const t={value:Y(e),illegal:!1,relevance:0,_top:l,_emitter:new p.__emitter(p)}
;return t._emitter.addText(e),t})(e),s=t.filter(O).filter(k).map((t=>E(t,e,!1)))
;s.unshift(n);const o=s.sort(((e,t)=>{
if(e.relevance!==t.relevance)return t.relevance-e.relevance
;if(e.language&&t.language){if(O(e.language).supersetOf===t.language)return 1
;if(O(t.language).supersetOf===e.language)return-1}return 0})),[r,a]=o,c=r
;return c.secondBest=a,c}function w(e){let t=null;const n=(e=>{
let t=e.className+" ";t+=e.parentNode?e.parentNode.className:""
;const n=p.languageDetectRe.exec(t);if(n){const t=O(n[1])
;return t||(X(a.replace("{}",n[1])),
X("Falling back to no-highlight mode for this block.",e)),t?n[1]:"no-highlight"}
return t.split(/\s+/).find((e=>b(e)||O(e)))})(e);if(b(n))return
;if(N("before:highlightElement",{el:e,language:n
}),e.dataset.highlighted)return void console.log("Element previously highlighted. To highlight again, first unset `dataset.highlighted`.",e)
;if(e.children.length>0&&(p.ignoreUnescapedHTML||(console.warn("One of your code blocks includes unescaped HTML. This is a potentially serious security risk."),
console.warn("https://github.com/highlightjs/highlight.js/wiki/security"),
console.warn("The element with unescaped HTML:"),
console.warn(e)),p.throwUnescapedHTML))throw new J("One of your code blocks includes unescaped HTML.",e.innerHTML)
;t=e;const i=t.textContent,o=n?m(i,{language:n,ignoreIllegals:!0}):x(i)
;e.innerHTML=o.value,e.dataset.highlighted="yes",((e,t,n)=>{const i=t&&s[t]||n
;e.classList.add("hljs"),e.classList.add("language-"+i)
})(e,n,o.language),e.result={language:o.language,re:o.relevance,
relevance:o.relevance},o.secondBest&&(e.secondBest={
language:o.secondBest.language,relevance:o.secondBest.relevance
}),N("after:highlightElement",{el:e,result:o,text:i})}let y=!1;function _(){
"loading"!==document.readyState?document.querySelectorAll(p.cssSelector).forEach(w):y=!0
}function O(e){return e=(e||"").toLowerCase(),i[e]||i[s[e]]}
function v(e,{languageName:t}){"string"==typeof e&&(e=[e]),e.forEach((e=>{
s[e.toLowerCase()]=t}))}function k(e){const t=O(e)
;return t&&!t.disableAutodetect}function N(e,t){const n=e;o.forEach((e=>{
e[n]&&e[n](t)}))}
"undefined"!=typeof window&&window.addEventListener&&window.addEventListener("DOMContentLoaded",(()=>{
y&&_()}),!1),Object.assign(n,{highlight:m,highlightAuto:x,highlightAll:_,
highlightElement:w,
highlightBlock:e=>(G("10.7.0","highlightBlock will be removed entirely in v12.0"),
G("10.7.0","Please use highlightElement now."),w(e)),configure:e=>{p=Q(p,e)},
initHighlighting:()=>{
_(),G("10.6.0","initHighlighting() deprecated.  Use highlightAll() now.")},
initHighlightingOnLoad:()=>{
_(),G("10.6.0","initHighlightingOnLoad() deprecated.  Use highlightAll() now.")
},registerLanguage:(e,t)=>{let s=null;try{s=t(n)}catch(t){
if(W("Language definition for '{}' could not be registered.".replace("{}",e)),
!r)throw t;W(t),s=l}
s.name||(s.name=e),i[e]=s,s.rawDefinition=t.bind(null,n),s.aliases&&v(s.aliases,{
languageName:e})},unregisterLanguage:e=>{delete i[e]
;for(const t of Object.keys(s))s[t]===e&&delete s[t]},
listLanguages:()=>Object.keys(i),getLanguage:O,registerAliases:v,
autoDetection:k,inherit:Q,addPlugin:e=>{(e=>{
e["before:highlightBlock"]&&!e["before:highlightElement"]&&(e["before:highlightElement"]=t=>{
e["before:highlightBlock"](Object.assign({block:t.el},t))
}),e["after:highlightBlock"]&&!e["after:highlightElement"]&&(e["after:highlightElement"]=t=>{
e["after:highlightBlock"](Object.assign({block:t.el},t))})})(e),o.push(e)},
removePlugin:e=>{const t=o.indexOf(e);-1!==t&&o.splice(t,1)}}),n.debugMode=()=>{
r=!1},n.safeMode=()=>{r=!0},n.versionString="11.10.0",n.regex={concat:h,
lookahead:g,either:f,optional:d,anyNumberOfTimes:u}
;for(const t in j)"object"==typeof j[t]&&e(j[t]);return Object.assign(n,j),n
},ne=te({});return ne.newInstance=()=>te({}),ne}()
;"object"==typeof exports&&"undefined"!=typeof module&&(module.exports=hljs);/*! `bash` grammar compiled for Highlight.js 11.10.0 */
(()=>{var e=(()=>{"use strict";return e=>{const s=e.regex,t={},n={begin:/\$\{/,
end:/\}/,contains:["self",{begin:/:-/,contains:[t]}]};Object.assign(t,{
className:"variable",variants:[{
begin:s.concat(/\$[\w\d#@][\w\d_]*/,"(?![\\w\\d])(?![$])")},n]});const a={
className:"subst",begin:/\$\(/,end:/\)/,contains:[e.BACKSLASH_ESCAPE]
},i=e.inherit(e.COMMENT(),{match:[/(^|\s)/,/#.*$/],scope:{2:"comment"}}),c={
begin:/<<-?\s*(?=\w+)/,starts:{contains:[e.END_SAME_AS_BEGIN({begin:/(\w+)/,
end:/(\w+)/,className:"string"})]}},o={className:"string",begin:/"/,end:/"/,
contains:[e.BACKSLASH_ESCAPE,t,a]};a.contains.push(o);const r={begin:/\$?\(\(/,
end:/\)\)/,contains:[{begin:/\d+#[0-9a-f]+/,className:"number"},e.NUMBER_MODE,t]
},l=e.SHEBANG({binary:"(fish|bash|zsh|sh|csh|ksh|tcsh|dash|scsh)",relevance:10
}),m={className:"function",begin:/\w[\w\d_]*\s*\(\s*\)\s*\{/,returnBegin:!0,
contains:[e.inherit(e.TITLE_MODE,{begin:/\w[\w\d_]*/})],relevance:0};return{
name:"Bash",aliases:["sh","zsh"],keywords:{$pattern:/\b[a-z][a-z0-9._-]+\b/,
keyword:["if","then","else","elif","fi","for","while","until","in","do","done","case","esac","function","select"],
literal:["true","false"],
built_in:["break","cd","continue","eval","exec","exit","export","getopts","hash","pwd","readonly","return","shift","test","times","trap","umask","unset","alias","bind","builtin","caller","command","declare","echo","enable","help","let","local","logout","mapfile","printf","read","readarray","source","sudo","type","typeset","ulimit","unalias","set","shopt","autoload","bg","bindkey","bye","cap","chdir","clone","comparguments","compcall","compctl","compdescribe","compfiles","compgroups","compquote","comptags","comptry","compvalues","dirs","disable","disown","echotc","echoti","emulate","fc","fg","float","functions","getcap","getln","history","integer","jobs","kill","limit","log","noglob","popd","print","pushd","pushln","rehash","sched","setcap","setopt","stat","suspend","ttyctl","unfunction","unhash","unlimit","unsetopt","vared","wait","whence","where","which","zcompile","zformat","zftp","zle","zmodload","zparseopts","zprof","zpty","zregexparse","zsocket","zstyle","ztcp","chcon","chgrp","chown","chmod","cp","dd","df","dir","dircolors","ln","ls","mkdir","mkfifo","mknod","mktemp","mv","realpath","rm","rmdir","shred","sync","touch","truncate","vdir","b2sum","base32","base64","cat","cksum","comm","csplit","cut","expand","fmt","fold","head","join","md5sum","nl","numfmt","od","paste","ptx","pr","sha1sum","sha224sum","sha256sum","sha384sum","sha512sum","shuf","sort","split","sum","tac","tail","tr","tsort","unexpand","uniq","wc","arch","basename","chroot","date","dirname","du","echo","env","expr","factor","groups","hostid","id","link","logname","nice","nohup","nproc","pathchk","pinky","printenv","printf","pwd","readlink","runcon","seq","sleep","stat","stdbuf","stty","tee","test","timeout","tty","uname","unlink","uptime","users","who","whoami","yes"]
},contains:[l,e.SHEBANG(),m,r,i,c,{match:/(\/[a-z._-]+)+/},o,{match:/\\"/},{
className:"string",begin:/'/,end:/'/},{match:/\\'/},t]}}})()
;hljs.registerLanguage("bash",e)})();/*! `diff` grammar compiled for Highlight.js 11.10.0 */
(()=>{var e=(()=>{"use strict";return e=>{const a=e.regex;return{name:"Diff",
aliases:["patch"],contains:[{className:"meta",relevance:10,
match:a.either(/^@@ +-\d+,\d+ +\+\d+,\d+ +@@/,/^\*\*\* +\d+,\d+ +\*\*\*\*$/,/^--- +\d+,\d+ +----$/)
},{className:"comment",variants:[{
begin:a.either(/Index: /,/^index/,/={3,}/,/^-{3}/,/^\*{3} /,/^\+{3}/,/^diff --git/),
end:/$/},{match:/^\*{15}$/}]},{className:"addition",begin:/^\+/,end:/$/},{
className:"deletion",begin:/^-/,end:/$/},{className:"addition",begin:/^!/,
end:/$/}]}}})();hljs.registerLanguage("diff",e)})();/*! `django` grammar compiled for Highlight.js 11.10.0 */
(()=>{var e=(()=>{"use strict";return e=>{const t={begin:/\|[A-Za-z]+:?/,
keywords:{
name:"truncatewords removetags linebreaksbr yesno get_digit timesince random striptags filesizeformat escape linebreaks length_is ljust rjust cut urlize fix_ampersands title floatformat capfirst pprint divisibleby add make_list unordered_list urlencode timeuntil urlizetrunc wordcount stringformat linenumbers slice date dictsort dictsortreversed default_if_none pluralize lower join center default truncatewords_html upper length phone2numeric wordwrap time addslashes slugify first escapejs force_escape iriencode last safe safeseq truncatechars localize unlocalize localtime utc timezone"
},contains:[e.QUOTE_STRING_MODE,e.APOS_STRING_MODE]};return{name:"Django",
aliases:["jinja"],case_insensitive:!0,subLanguage:"xml",
contains:[e.COMMENT(/\{%\s*comment\s*%\}/,/\{%\s*endcomment\s*%\}/),e.COMMENT(/\{#/,/#\}/),{
className:"template-tag",begin:/\{%/,end:/%\}/,contains:[{className:"name",
begin:/\w+/,keywords:{
name:"comment endcomment load templatetag ifchanged endifchanged if endif firstof for endfor ifnotequal endifnotequal widthratio extends include spaceless endspaceless regroup ifequal endifequal ssi now with cycle url filter endfilter debug block endblock else autoescape endautoescape csrf_token empty elif endwith static trans blocktrans endblocktrans get_static_prefix get_media_prefix plural get_current_language language get_available_languages get_current_language_bidi get_language_info get_language_info_list localize endlocalize localtime endlocaltime timezone endtimezone get_current_timezone verbatim"
},starts:{endsWithParent:!0,keywords:"in by as",contains:[t],relevance:0}}]},{
className:"template-variable",begin:/\{\{/,end:/\}\}/,contains:[t]}]}}})()
;hljs.registerLanguage("django",e)})();/*! `json` grammar compiled for Highlight.js 11.10.0 */
(()=>{var e=(()=>{"use strict";return e=>{const a=["true","false","null"],s={
scope:"literal",beginKeywords:a.join(" ")};return{name:"JSON",aliases:["jsonc"],
keywords:{literal:a},contains:[{className:"attr",
begin:/"(\\.|[^\\"\r\n])*"(?=\s*:)/,relevance:1.01},{match:/[{}[\],:]/,
className:"punctuation",relevance:0
},e.QUOTE_STRING_MODE,s,e.C_NUMBER_MODE,e.C_LINE_COMMENT_MODE,e.C_BLOCK_COMMENT_MODE],
illegal:"\\S"}}})();hljs.registerLanguage("json",e)})();/*! `markdown` grammar compiled for Highlight.js 11.10.0 */
(()=>{var e=(()=>{"use strict";return e=>{const n={begin:/<\/?[A-Za-z_]/,
end:">",subLanguage:"xml",relevance:0},a={variants:[{begin:/\[.+?\]\[.*?\]/,
relevance:0},{
begin:/\[.+?\]\(((data|javascript|mailto):|(?:http|ftp)s?:\/\/).*?\)/,
relevance:2},{
begin:e.regex.concat(/\[.+?\]\(/,/[A-Za-z][A-Za-z0-9+.-]*/,/:\/\/.*?\)/),
relevance:2},{begin:/\[.+?\]\([./?&#].*?\)/,relevance:1},{
begin:/\[.*?\]\(.*?\)/,relevance:0}],returnBegin:!0,contains:[{match:/\[(?=\])/
},{className:"string",relevance:0,begin:"\\[",end:"\\]",excludeBegin:!0,
returnEnd:!0},{className:"link",relevance:0,begin:"\\]\\(",end:"\\)",
excludeBegin:!0,excludeEnd:!0},{className:"symbol",relevance:0,begin:"\\]\\[",
end:"\\]",excludeBegin:!0,excludeEnd:!0}]},i={className:"strong",contains:[],
variants:[{begin:/_{2}(?!\s)/,end:/_{2}/},{begin:/\*{2}(?!\s)/,end:/\*{2}/}]
},s={className:"emphasis",contains:[],variants:[{begin:/\*(?![*\s])/,end:/\*/},{
begin:/_(?![_\s])/,end:/_/,relevance:0}]},c=e.inherit(i,{contains:[]
}),t=e.inherit(s,{contains:[]});i.contains.push(t),s.contains.push(c)
;let g=[n,a];return[i,s,c,t].forEach((e=>{e.contains=e.contains.concat(g)
})),g=g.concat(i,s),{name:"Markdown",aliases:["md","mkdown","mkd"],contains:[{
className:"section",variants:[{begin:"^#{1,6}",end:"$",contains:g},{
begin:"(?=^.+?\\n[=-]{2,}$)",contains:[{begin:"^[=-]*$"},{begin:"^",end:"\\n",
contains:g}]}]},n,{className:"bullet",begin:"^[ \t]*([*+-]|(\\d+\\.))(?=\\s+)",
end:"\\s+",excludeEnd:!0},i,s,{className:"quote",begin:"^>\\s+",contains:g,
end:"$"},{className:"code",variants:[{begin:"(`{3,})[^`](.|\\n)*?\\1`*[ ]*"},{
begin:"(~{3,})[^~](.|\\n)*?\\1~*[ ]*"},{begin:"```",end:"```+[ ]*$"},{
begin:"~~~",end:"~~~+[ ]*$"},{begin:"`.+?`"},{begin:"(?=^( {4}|\\t))",
contains:[{begin:"^( {4}|\\t)",end:"(\\n)$"}],relevance:0}]},{
begin:"^[-\\*]{3,}",end:"$"},a,{begin:/^\[[^\n]+\]:/,returnBegin:!0,contains:[{
className:"symbol",begin:/\[/,end:/\]/,excludeBegin:!0,excludeEnd:!0},{
className:"link",begin:/:\s*/,end:/$/,excludeBegin:!0}]},{scope:"literal",
match:/&([a-zA-Z0-9]+|#[0-9]{1,7}|#[Xx][0-9a-fA-F]{1,6});/}]}}})()
;hljs.registerLanguage("markdown",e)})();/*! `rust` grammar compiled for Highlight.js 11.10.0 */
(()=>{var e=(()=>{"use strict";return e=>{
const t=e.regex,n=/(r#)?/,a=t.concat(n,e.UNDERSCORE_IDENT_RE),i=t.concat(n,e.IDENT_RE),r={
className:"title.function.invoke",relevance:0,
begin:t.concat(/\b/,/(?!let|for|while|if|else|match\b)/,i,t.lookahead(/\s*\(/))
},s="([ui](8|16|32|64|128|size)|f(32|64))?",l=["drop ","Copy","Send","Sized","Sync","Drop","Fn","FnMut","FnOnce","ToOwned","Clone","Debug","PartialEq","PartialOrd","Eq","Ord","AsRef","AsMut","Into","From","Default","Iterator","Extend","IntoIterator","DoubleEndedIterator","ExactSizeIterator","SliceConcatExt","ToString","assert!","assert_eq!","bitflags!","bytes!","cfg!","col!","concat!","concat_idents!","debug_assert!","debug_assert_eq!","env!","eprintln!","panic!","file!","format!","format_args!","include_bytes!","include_str!","line!","local_data_key!","module_path!","option_env!","print!","println!","select!","stringify!","try!","unimplemented!","unreachable!","vec!","write!","writeln!","macro_rules!","assert_ne!","debug_assert_ne!"],o=["i8","i16","i32","i64","i128","isize","u8","u16","u32","u64","u128","usize","f32","f64","str","char","bool","Box","Option","Result","String","Vec"]
;return{name:"Rust",aliases:["rs"],keywords:{$pattern:e.IDENT_RE+"!?",type:o,
keyword:["abstract","as","async","await","become","box","break","const","continue","crate","do","dyn","else","enum","extern","false","final","fn","for","if","impl","in","let","loop","macro","match","mod","move","mut","override","priv","pub","ref","return","self","Self","static","struct","super","trait","true","try","type","typeof","union","unsafe","unsized","use","virtual","where","while","yield"],
literal:["true","false","Some","None","Ok","Err"],built_in:l},illegal:"</",
contains:[e.C_LINE_COMMENT_MODE,e.COMMENT("/\\*","\\*/",{contains:["self"]
}),e.inherit(e.QUOTE_STRING_MODE,{begin:/b?"/,illegal:null}),{
className:"string",variants:[{begin:/b?r(#*)"(.|\n)*?"\1(?!#)/},{
begin:/b?'\\?(x\w{2}|u\w{4}|U\w{8}|.)'/}]},{className:"symbol",
begin:/'[a-zA-Z_][a-zA-Z0-9_]*/},{className:"number",variants:[{
begin:"\\b0b([01_]+)"+s},{begin:"\\b0o([0-7_]+)"+s},{
begin:"\\b0x([A-Fa-f0-9_]+)"+s},{
begin:"\\b(\\d[\\d_]*(\\.[0-9_]+)?([eE][+-]?[0-9_]+)?)"+s}],relevance:0},{
begin:[/fn/,/\s+/,a],className:{1:"keyword",3:"title.function"}},{
className:"meta",begin:"#!?\\[",end:"\\]",contains:[{className:"string",
begin:/"/,end:/"/,contains:[e.BACKSLASH_ESCAPE]}]},{
begin:[/let/,/\s+/,/(?:mut\s+)?/,a],className:{1:"keyword",3:"keyword",
4:"variable"}},{begin:[/for/,/\s+/,a,/\s+/,/in/],className:{1:"keyword",
3:"variable",5:"keyword"}},{begin:[/type/,/\s+/,a],className:{1:"keyword",
3:"title.class"}},{begin:[/(?:trait|enum|struct|union|impl|for)/,/\s+/,a],
className:{1:"keyword",3:"title.class"}},{begin:e.IDENT_RE+"::",keywords:{
keyword:"Self",built_in:l,type:o}},{className:"punctuation",begin:"->"},r]}}})()
;hljs.registerLanguage("rust",e)})();/*! `shell` grammar compiled for Highlight.js 11.10.0 */
(()=>{var s=(()=>{"use strict";return s=>({name:"Shell Session",
aliases:["console","shellsession"],contains:[{className:"meta.prompt",
begin:/^\s{0,3}[/~\w\d[\]()@-]*[>%$#][ ]?/,starts:{end:/[^\\](?=\s*$)/,
subLanguage:"bash"}}]})})();hljs.registerLanguage("shell",s)})();/*! `yaml` grammar compiled for Highlight.js 11.10.0 */
(()=>{var e=(()=>{"use strict";return e=>{
const n="true false yes no null",a="[\\w#;/?:@&=+$,.~*'()[\\]]+",s={
className:"string",relevance:0,variants:[{begin:/'/,end:/'/},{begin:/"/,end:/"/
},{begin:/\S+/}],contains:[e.BACKSLASH_ESCAPE,{className:"template-variable",
variants:[{begin:/\{\{/,end:/\}\}/},{begin:/%\{/,end:/\}/}]}]},i=e.inherit(s,{
variants:[{begin:/'/,end:/'/},{begin:/"/,end:/"/},{begin:/[^\s,{}[\]]+/}]}),l={
end:",",endsWithParent:!0,excludeEnd:!0,keywords:n,relevance:0},t={begin:/\{/,
end:/\}/,contains:[l],illegal:"\\n",relevance:0},g={begin:"\\[",end:"\\]",
contains:[l],illegal:"\\n",relevance:0},b=[{className:"attr",variants:[{
begin:/\w[\w :()\./-]*:(?=[ \t]|$)/},{begin:/"\w[\w :()\./-]*":(?=[ \t]|$)/},{
begin:/'\w[\w :()\./-]*':(?=[ \t]|$)/}]},{className:"meta",begin:"^---\\s*$",
relevance:10},{className:"string",
begin:"[\\|>]([1-9]?[+-])?[ ]*\\n( +)[^ ][^\\n]*\\n(\\2[^\\n]+\\n?)*"},{
begin:"<%[%=-]?",end:"[%-]?%>",subLanguage:"ruby",excludeBegin:!0,excludeEnd:!0,
relevance:0},{className:"type",begin:"!\\w+!"+a},{className:"type",
begin:"!<"+a+">"},{className:"type",begin:"!"+a},{className:"type",begin:"!!"+a
},{className:"meta",begin:"&"+e.UNDERSCORE_IDENT_RE+"$"},{className:"meta",
begin:"\\*"+e.UNDERSCORE_IDENT_RE+"$"},{className:"bullet",begin:"-(?=[ ]|$)",
relevance:0},e.HASH_COMMENT_MODE,{beginKeywords:n,keywords:{literal:n}},{
className:"number",
begin:"\\b[0-9]{4}(-[0-9][0-9]){0,2}([Tt \\t][0-9][0-9]?(:[0-9][0-9]){2})?(\\.[0-9]*)?([ \\t])*(Z|[-+][0-9][0-9]?(:[0-9][0-9])?)?\\b"
},{className:"number",begin:e.C_NUMBER_RE+"\\b",relevance:0},t,g,s],r=[...b]
;return r.pop(),r.push(i),l.contains=r,{name:"YAML",case_insensitive:!0,
aliases:["yml"],contains:b}}})();hljs.registerLanguage("yaml",e)})();
