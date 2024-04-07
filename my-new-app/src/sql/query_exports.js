var dbCon = require('./sqlite_connect.js');
var db = dbCon.db;


export const query_exports = {
  getprocesses: () => {
    const sql = 'SELECT * FROM processes';
    const processes = db.prepare(sql).all();
    return processes;
  }
};

