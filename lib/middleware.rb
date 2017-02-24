class Superconductor::Middleware
  def initialize(app)
    @app = app
  end

  def call(env)
    status, headers, body = @app.call(env)
    hello = Superconductor.hello_world
    hello.free = Superconductor[:hello_world_free]
    [status, headers, ["blah #{hello}"]]
  end
end
