local M = {}

local function get_database_url()
  local url = vim.fn.environ()["POLYQUERY_DATABASE_URL"]
  if url and url ~= "" then
    return url
  end
  local ok, result = pcall(vim.fn.inputsecret, "Database URL: ")
  if ok and result ~= "" then
    return result
  end
  return nil
end

function M.setup(opts)
  opts = opts or {}

  local server_path = opts.server_path or "polyquery"

  -- Start the LSP client
  vim.api.nvim_create_autocmd("FileType", {
    pattern = "*",
    callback = function(args)
      local bufnr = args.buf
      local clients = vim.lsp.get_clients({ name = "polyquery", bufnr = bufnr })
      if #clients > 0 then
        return
      end

      vim.lsp.start({
        name = "polyquery",
        cmd = { server_path },
        root_dir = vim.fn.getcwd(),
        on_attach = function(client, bufnr)
          local db_url = get_database_url()
          if db_url then
            client.config.settings = vim.tbl_deep_extend(
              "force",
              client.config.settings or {},
              { POLYQUERY_DATABASE_URL = db_url }
            )
          end
          vim.api.nvim_buf_set_option(bufnr, "omnifunc", "v:lua.vim.lsp.omnifunc")
        end,
        capabilities = vim.lsp.protocol.make_client_capabilities(),
      })
    end,
  })

  -- Run Query command
  vim.api.nvim_create_user_command("PolyqueryRun", function(info)
    local sql = info.args
    if sql == "" then
      -- Use visual selection or current line
      local mode = vim.fn.mode()
      if mode == "v" or mode == "V" then
        local start_pos = vim.fn.getpos("'<")
        local end_pos = vim.fn.getpos("'>")
        local lines = vim.api.nvim_buf_get_lines(0, start_pos[2] - 1, end_pos[2], false)
        sql = table.concat(lines, "\n")
      else
        sql = vim.api.nvim_get_current_line()
      end
    end

    if sql == "" then
      vim.notify("No SQL to execute", vim.log.levels.WARN)
      return
    end

    local clients = vim.lsp.get_clients({ name = "polyquery", bufnr = 0 })
    if #clients == 0 then
      vim.notify("Polyquery LSP not running", vim.log.levels.ERROR)
      return
    end

    local client = clients[1]
    client.request("workspace/executeCommand", {
      command = "polyquery.runQuery",
      arguments = { vim.uri_from_bufnr(0), sql },
    }, function(err, result)
      if err then
        vim.notify("Query error: " .. vim.inspect(err), vim.log.levels.ERROR)
        return
      end
      if result.type == "error" then
        vim.notify(result.message, vim.log.levels.ERROR)
      elseif result.text then
        vim.api.nvim_command("new")
        vim.api.nvim_buf_set_name(0, "polyquery://results")
        vim.api.nvim_buf_set_lines(0, 0, -1, false, vim.split(result.text, "\n"))
        vim.api.nvim_buf_set_option(0, "buftype", "nofile")
        vim.api.nvim_buf_set_option(0, "readonly", true)
      end
    end, bufnr)
  end, { nargs = "?", desc = "Execute SQL query via Polyquery" })
end

function M.run_query(sql)
  vim.cmd("PolyqueryRun " .. (sql or ""))
end

return M
