# examination

一个简单的考试命令行工具，通过读取本地考卷json文件生成考试题目，支持用户通过命令行进行作答，最终提交考卷后可支持计算得分。

## 特性

1. 支持本地考卷文件读取，支持自定义考卷文件路径
2. 支持单选题，后续将支持多选题，填空题，判断题等
3. 支持自动计算分数
4. 支持错误题目对比
5. 支持自定义退出（Ctrl+c）和交卷快捷键（Ctrl+s）

## 安装

```shell
cargo install examination
```

## 运行

```shell
examination
```

## 配置

- 默认数据路径为 `.data`，可以通过环境变量 `EXAMINATION_PATH` 来修改
- 默认考卷文件路径为 `.data/question.json`
- 默认配置路径为 `.config`，可以通过环境变量 `EXAMINATION_CONFIG` 来修改
- 默认快捷键配置文件为 `.config/config.json5`

## 考卷文件格式

参考 [question.json](./.data/question.json)

## Roadmap

1. 支持多选，填空题，判断题
2. 支持自定义题目得分 
3. 支持考试计时 
4. 支持从题库中随机出题 
5. 支持错题记录 
6. 支持错题重做



