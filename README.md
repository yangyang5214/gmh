

> gmh (git commit message helper) 

### Usage

https://platform.deepseek.com/api_keys


- config OPENAI_API_KEY

```
export OPENAI_API_KEY=sk-xxx
```

- run gmh

```
Some file has been modified
then
git add .

# run gmh
➜  gmh git:(master) ✗ gmh
Generated commit message:
Update README.md with usage instructions for gmh tool
Do you want to commit these changes? (y/n)
y
[master 8099ff2] Update README.md with usage instructions for gmh tool
 1 file changed, 17 insertions(+)
Changes committed successfully.
```