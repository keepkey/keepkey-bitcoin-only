# Debugging Strategy Improvements

## Quick Debugging Checklist

### 1. Error Analysis Phase (First 2 minutes)
- [ ] Read the EXACT error message
- [ ] Note the file path and line number
- [ ] Identify error type: Import/Syntax/Runtime/Type
- [ ] Confirm which project is affected

### 2. Context Verification Phase (Next 1 minute)
- [ ] Verify the project directory
- [ ] Check running port/URL matches expected project
- [ ] Confirm framework version (React/Chakra UI/etc.)
- [ ] Note any recent changes or context

### 3. Pattern Recognition Phase (Next 2 minutes)
- [ ] Search for similar component usage in the project
- [ ] Check imports in nearby files
- [ ] Look for established patterns
- [ ] Verify component is not already imported

### 4. Solution Implementation Phase
- [ ] Start with the simplest fix
- [ ] Make one change at a time
- [ ] Test after each change
- [ ] Document what worked

## Common React Error Patterns

### Import Errors
```
Failed to resolve import "X" from "Y"
```
**Quick Fix Sequence:**
1. Check if X is from a node_module (npm package)
2. Check if X exists at the relative path
3. Check other files for correct import pattern
4. Verify the component name and export type

### Component Rendering Errors
```
Element type is invalid: expected a string... but got: object
```
**Quick Fix Sequence:**
1. Check the import statement
2. Verify named vs default export
3. Ensure component is exported
4. Check for circular dependencies

### Module Not Found
```
Module not found: Can't resolve 'X'
```
**Quick Fix Sequence:**
1. Run `npm install` if it's a package
2. Check package.json for the dependency
3. Verify the import path
4. Check for typos in import statement

## Grep Commands for Quick Debugging

```bash
# Find how a component is used elsewhere
grep -r "ComponentName" --include="*.tsx" --include="*.ts" .

# Find import patterns for a package
grep -r "from [@'\"]chakra-ui" --include="*.tsx" .

# Find all Progress component usage
grep -r "Progress" --include="*.tsx" -B2 -A2 .

# Find specific import patterns
grep -r "import.*Progress.*from" --include="*.tsx" .
```

## Project Structure Quick Reference

When debugging, quickly identify:
1. **UI Components Location**: Usually in `/components/ui/` or from UI library
2. **Context Providers**: Usually in `/contexts/`
3. **Types**: Usually in `/types/`
4. **Assets**: Usually in `/assets/`

## Time-Boxing Strategy

- **0-5 minutes**: Simple import/syntax fixes
- **5-10 minutes**: Component integration issues
- **10-15 minutes**: Complex state/props issues
- **15+ minutes**: Architectural problems - step back and reconsider

## Red Flags to Watch For

1. **Making changes in multiple projects** - Stop and verify which one is correct
2. **Creating new components to fix errors** - Usually not necessary
3. **Modifying core infrastructure** - Simple errors rarely need this
4. **Import paths with many ../../../** - Consider absolute imports
5. **Assuming custom components** - Check if it's from the UI library first

## The "Occam's Razor" Debugging Principle

The simplest explanation is usually correct:
- Import error? Wrong import path
- Component not rendering? Missing or incorrect import
- Type error? Props mismatch
- Module not found? Not installed or wrong name

## Post-Fix Verification

After fixing an issue:
1. Document the fix
2. Check for similar issues elsewhere
3. Consider adding a lint rule
4. Update team knowledge base
5. Create a unit test if applicable