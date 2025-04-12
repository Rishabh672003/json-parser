**All the terminals are all capital**

## First Attempt, Trying to wing the grammar myself

After reading the actual spec this is just purely wrong, I will leave it here for archival purposes

```plaintext
json     := obj
obj      := OCB, pair*, CCB
pair     := STR_LIT, COLON, value
value    := obj | STR_LIT | NUMBER | array | TRUE | FALSE | NULL
array    := OSB, [ elements ]*, CSB
elements := value, [ COMMA, value ]\*
```

## Now let me try to put the Mckeeman grammar from the json.org

```plaintext
Json     := Element
Value    := Object | Array | STR_LIT | NUMBER | TRUE | FALSE | NULL
Object   := OCB, Members, CSB
Members  := Member | Member, COMMA, Member
Member   := STR_LIT, COLON, Element
Array    := OSB, Elements, CSB
Elements := Element | Element, COMMA, Elements
Element  := Value
```
