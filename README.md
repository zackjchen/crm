# rpc study
a sample crm system


## Postgres
```sql
-- 查看表的index大小
select pg_size_pretty(pg_indexes_size('user_stats'));

-- 查看表的数据大小
select pg_size_pretty(pg_relation_size('user_stats'));
```


## Protocbuf
proto导入本地文件解决vs code报错的问题
1. 创建.vscode目录
2. 在.vscode下创建setting.json
3. 写入内容
```json
{
    "protoc": {
        "options": [
            "--proto_path=protos/common,protos/crm,protos/user_stats",
        ]
    }
}
```
