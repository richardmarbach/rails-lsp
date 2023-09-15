local id = nil

local function attach_lsp(args)
	if id == nil then
		return
	end

	local bufnr = args.buffer or args.buf
	if not bufnr then
		return
	end

	if not vim.lsp.buf_is_attached(args.buffer, id) then
		vim.lsp.buf_attach_client(args.buffer, id)
	end
end

local function start_rails_lsp()
	if id ~= nil then
		return
	end

	id = vim.lsp.start_client({
		name = "rails-ls",
		cmd = { "rails-lsp" },
		root_dir = vim.loop.cwd() .. "/rails-sample",
	})

	local bufnr = vim.api.nvim_get_current_buf()

	local filetype = vim.api.nvim_buf_get_option(bufnr, "filetype")
	if filetype ~= "ruby" then
		return
	end

	attach_lsp({ buffer = bufnr })
end

local function stop_rails_lsp()
	if id == nil then
		return
	end
	vim.lsp.stop_client(id, { force = true })
	id = nil
end

_G.restart_rails_lsp = function()
	stop_rails_lsp()
	start_rails_lsp()
end

vim.api.nvim_create_autocmd({ "BufNew", "BufEnter" }, {
	group = vim.api.nvim_create_augroup("Rails-ls", {}),
	pattern = { "*.rb" },
	callback = attach_lsp,
})
