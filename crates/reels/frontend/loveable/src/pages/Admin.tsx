import { useState, useEffect } from 'react';
import { Navigation } from '@/components/Navigation';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Users, Video, DollarSign, TrendingUp } from 'lucide-react';
import { useAuth } from '@/hooks/useAuth';
import { useNavigate } from 'react-router-dom';
import { supabase } from '@/integrations/supabase/client';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Badge } from '@/components/ui/badge';

const Admin = () => {
  const { user, loading } = useAuth();
  const navigate = useNavigate();
  const [isAdmin, setIsAdmin] = useState(false);
  const [stats, setStats] = useState({
    totalUsers: 0,
    totalVideos: 0,
    totalCreditsUsed: 0,
    totalViews: 0,
  });
  const [users, setUsers] = useState<any[]>([]);

  useEffect(() => {
    if (!loading && !user) {
      navigate('/auth');
    } else if (user) {
      checkAdminAccess();
    }
  }, [user, loading, navigate]);

  const checkAdminAccess = async () => {
    if (!user) return;

    const { data } = await supabase
      .from('user_roles')
      .select('role')
      .eq('user_id', user.id)
      .eq('role', 'admin')
      .maybeSingle();

    if (!data) {
      navigate('/dashboard');
    } else {
      setIsAdmin(true);
      loadAdminData();
    }
  };

  const loadAdminData = async () => {
    // Load stats
    const [profilesData, videosData, creditsData] = await Promise.all([
      supabase.from('profiles').select('id'),
      supabase.from('video_pages').select('credits_used, views'),
      supabase.from('credits').select('total_spent'),
    ]);

    const totalCreditsUsed = creditsData.data?.reduce((sum, c) => sum + (c.total_spent || 0), 0) || 0;
    const totalViews = videosData.data?.reduce((sum, v) => sum + (v.views || 0), 0) || 0;

    setStats({
      totalUsers: profilesData.data?.length || 0,
      totalVideos: videosData.data?.length || 0,
      totalCreditsUsed,
      totalViews,
    });

    // Load users with their credits
    const { data: usersData } = await supabase
      .from('profiles')
      .select(`
        id,
        email,
        full_name,
        created_at,
        credits (
          balance,
          total_spent
        )
      `)
      .order('created_at', { ascending: false })
      .limit(10);

    setUsers(usersData || []);
  };

  if (loading || !isAdmin) {
    return <div className="min-h-screen bg-background" />;
  }

  return (
    <div className="min-h-screen bg-background">
      <Navigation />
      
      <div className="pt-24 px-6 pb-12">
        <div className="container mx-auto max-w-7xl">
          {/* Header */}
          <div className="mb-8">
            <h1 className="text-4xl font-bold mb-2">Admin Dashboard</h1>
            <p className="text-muted-foreground">Manage users and monitor platform activity</p>
          </div>

          {/* Stats Grid */}
          <div className="grid md:grid-cols-4 gap-6 mb-8">
            <StatsCard
              icon={<Users className="w-6 h-6" />}
              title="Total Users"
              value={stats.totalUsers.toString()}
              gradient="gradient-primary"
            />
            <StatsCard
              icon={<Video className="w-6 h-6" />}
              title="Total Videos"
              value={stats.totalVideos.toString()}
              gradient="gradient-accent"
            />
            <StatsCard
              icon={<DollarSign className="w-6 h-6" />}
              title="Credits Used"
              value={stats.totalCreditsUsed.toString()}
              gradient="gradient-primary"
            />
            <StatsCard
              icon={<TrendingUp className="w-6 h-6" />}
              title="Total Views"
              value={stats.totalViews.toString()}
              gradient="gradient-accent"
            />
          </div>

          {/* Recent Users Table */}
          <Card className="glass-card">
            <CardHeader>
              <CardTitle>Recent Users</CardTitle>
            </CardHeader>
            <CardContent>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Name</TableHead>
                    <TableHead>Email</TableHead>
                    <TableHead>Credits</TableHead>
                    <TableHead>Total Spent</TableHead>
                    <TableHead>Joined</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {users.map((user) => (
                    <TableRow key={user.id}>
                      <TableCell className="font-medium">{user.full_name || 'N/A'}</TableCell>
                      <TableCell>{user.email}</TableCell>
                      <TableCell>
                        <Badge variant="secondary">
                          {user.credits?.balance || 0}
                        </Badge>
                      </TableCell>
                      <TableCell>{user.credits?.total_spent || 0}</TableCell>
                      <TableCell className="text-muted-foreground">
                        {new Date(user.created_at).toLocaleDateString()}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
};

const StatsCard = ({ icon, title, value, gradient }: { icon: React.ReactNode; title: string; value: string; gradient: string }) => (
  <Card className="glass-card">
    <CardHeader className="flex flex-row items-center justify-between pb-2">
      <CardTitle className="text-sm font-medium text-muted-foreground">{title}</CardTitle>
      <div className={`w-10 h-10 rounded-lg ${gradient} flex items-center justify-center text-white`}>
        {icon}
      </div>
    </CardHeader>
    <CardContent>
      <div className="text-3xl font-bold">{value}</div>
    </CardContent>
  </Card>
);

export default Admin;